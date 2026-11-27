-- OpenScout Intelligence: Initial PostgreSQL Schema
-- Hybrid Relational + JSONB event model for StatsBomb 120x80 coordinates

CREATE TABLE IF NOT EXISTS competitions (
    id SERIAL PRIMARY KEY,
    competition_id INT NOT NULL,
    season_id INT NOT NULL,
    competition_name VARCHAR(120) NOT NULL,
    season_name VARCHAR(60) NOT NULL,
    country_name VARCHAR(80),
    CONSTRAINT uq_comp_season UNIQUE (competition_id, season_id)
);

CREATE TABLE IF NOT EXISTS teams (
    id INT PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    country VARCHAR(80)
);

CREATE INDEX IF NOT EXISTS ix_teams_name ON teams (name);

CREATE TABLE IF NOT EXISTS players (
    id INT PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    primary_position VARCHAR(60)
);

CREATE INDEX IF NOT EXISTS ix_players_name ON players (name);

CREATE TABLE IF NOT EXISTS matches (
    id INT PRIMARY KEY,
    competition_pk INT NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    match_date DATE,
    home_team_id INT NOT NULL REFERENCES teams(id),
    away_team_id INT NOT NULL REFERENCES teams(id),
    home_score INT NOT NULL DEFAULT 0,
    away_score INT NOT NULL DEFAULT 0,
    stadium VARCHAR(150)
);

CREATE INDEX IF NOT EXISTS ix_matches_competition ON matches (competition_pk);

CREATE TABLE IF NOT EXISTS match_events (
    id UUID PRIMARY KEY,
    match_id INT NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    event_index INT NOT NULL,
    period INT NOT NULL,
    minute INT NOT NULL,
    second INT NOT NULL,
    possession INT NOT NULL,
    team_id INT NOT NULL REFERENCES teams(id),
    player_id INT REFERENCES players(id),
    event_type VARCHAR(60) NOT NULL,
    outcome VARCHAR(60),
    location_x DOUBLE PRECISION,
    location_y DOUBLE PRECISION,
    end_x DOUBLE PRECISION,
    end_y DOUBLE PRECISION,
    xg DOUBLE PRECISION,
    extra_data JSONB
);

CREATE INDEX IF NOT EXISTS ix_events_match_timeline ON match_events (match_id, period, event_index);
CREATE INDEX IF NOT EXISTS ix_events_player_type ON match_events (player_id, event_type);
CREATE INDEX IF NOT EXISTS ix_events_type ON match_events (event_type);

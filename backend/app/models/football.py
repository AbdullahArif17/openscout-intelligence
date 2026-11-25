from datetime import date
from uuid import UUID

from sqlalchemy import Date, Float, ForeignKey, Index, Integer, String, UniqueConstraint
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.dialects.postgresql import UUID as PG_UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.core.database import Base


class Competition(Base):
    """Football competition and season metadata from StatsBomb."""

    __tablename__ = "competitions"
    __table_args__ = (
        UniqueConstraint("competition_id", "season_id", name="uq_comp_season"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    competition_id: Mapped[int] = mapped_column(Integer, nullable=False)
    season_id: Mapped[int] = mapped_column(Integer, nullable=False)
    competition_name: Mapped[str] = mapped_column(String(120), nullable=False)
    season_name: Mapped[str] = mapped_column(String(60), nullable=False)
    country_name: Mapped[str | None] = mapped_column(String(80), nullable=True)

    matches: Mapped[list["Match"]] = relationship(back_populates="competition")


class Team(Base):
    """Football club or national team."""

    __tablename__ = "teams"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(120), nullable=False, index=True)
    country: Mapped[str | None] = mapped_column(String(80), nullable=True)


class Player(Base):
    """Individual football player."""

    __tablename__ = "players"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(150), nullable=False, index=True)
    primary_position: Mapped[str | None] = mapped_column(String(60), nullable=True)


class Match(Base):
    """Single football match with scoreline and team references."""

    __tablename__ = "matches"

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    competition_pk: Mapped[int] = mapped_column(
        ForeignKey("competitions.id", ondelete="CASCADE"), nullable=False, index=True
    )
    match_date: Mapped[date | None] = mapped_column(Date, nullable=True)
    home_team_id: Mapped[int] = mapped_column(
        ForeignKey("teams.id"), nullable=False, index=True
    )
    away_team_id: Mapped[int] = mapped_column(
        ForeignKey("teams.id"), nullable=False, index=True
    )
    home_score: Mapped[int] = mapped_column(Integer, default=0)
    away_score: Mapped[int] = mapped_column(Integer, default=0)
    stadium: Mapped[str | None] = mapped_column(String(150), nullable=True)

    competition: Mapped["Competition"] = relationship(back_populates="matches")
    events: Mapped[list["MatchEvent"]] = relationship(
        back_populates="match", cascade="all, delete-orphan"
    )


class MatchEvent(Base):
    """Atomic match event (Pass, Shot, Pressure, Carry) using StatsBomb 120x80 pitch coordinates."""

    __tablename__ = "match_events"
    __table_args__ = (
        Index("ix_events_match_timeline", "match_id", "period", "index"),
        Index("ix_events_player_type", "player_id", "event_type"),
    )

    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True)
    match_id: Mapped[int] = mapped_column(
        ForeignKey("matches.id", ondelete="CASCADE"), nullable=False
    )
    index: Mapped[int] = mapped_column(Integer, nullable=False)
    period: Mapped[int] = mapped_column(Integer, nullable=False)
    minute: Mapped[int] = mapped_column(Integer, nullable=False)
    second: Mapped[int] = mapped_column(Integer, nullable=False)
    possession: Mapped[int] = mapped_column(Integer, nullable=False)

    team_id: Mapped[int] = mapped_column(
        ForeignKey("teams.id"), nullable=False, index=True
    )
    player_id: Mapped[int | None] = mapped_column(
        ForeignKey("players.id"), nullable=True
    )
    event_type: Mapped[str] = mapped_column(String(60), nullable=False, index=True)
    outcome: Mapped[str | None] = mapped_column(String(60), nullable=True)

    # StatsBomb standard pitch: X in [0, 120], Y in [0, 80]
    location_x: Mapped[float | None] = mapped_column(Float, nullable=True)
    location_y: Mapped[float | None] = mapped_column(Float, nullable=True)
    end_x: Mapped[float | None] = mapped_column(Float, nullable=True)
    end_y: Mapped[float | None] = mapped_column(Float, nullable=True)
    xg: Mapped[float | None] = mapped_column(Float, nullable=True)

    # Flexible storage for event-specific attributes (pass_height, body_part, freeze_frame, etc.)
    extra_data: Mapped[dict | None] = mapped_column(JSONB, nullable=True)

    match: Mapped["Match"] = relationship(back_populates="events")

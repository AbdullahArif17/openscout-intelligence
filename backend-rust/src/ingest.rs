use crate::models::{PlayerMatchSummary, StatsBombEvent};
use sqlx::PgPool;
use std::collections::HashMap;

const STATSBOMB_EVENTS_BASE: &str =
    "https://raw.githubusercontent.com/statsbomb/open-data/master/data/events";

/// Fetch and deserialize all StatsBomb events for a given match ID.
pub async fn fetch_statsbomb_events(match_id: i32) -> anyhow::Result<Vec<StatsBombEvent>> {
    let url = format!("{STATSBOMB_EVENTS_BASE}/{match_id}.json");
    let client = reqwest::Client::new();
    let events = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<StatsBombEvent>>()
        .await?;
    Ok(events)
}

/// Aggregate per-player shooting and passing metrics from raw StatsBomb events.
pub fn summarize_players(events: &[StatsBombEvent]) -> Vec<PlayerMatchSummary> {
    struct Acc {
        name: String,
        team: String,
        shots: u32,
        goals: u32,
        total_xg: f64,
        passes_attempted: u32,
        passes_completed: u32,
    }

    let mut map: HashMap<i32, Acc> = HashMap::new();

    for ev in events {
        // Exclude penalty shootout (period 5) for open-play/in-match xG accuracy
        if ev.period > 4 {
            continue;
        }
        let Some(player) = &ev.player else {
            continue;
        };

        let entry = map.entry(player.id).or_insert_with(|| Acc {
            name: player.name.clone(),
            team: ev.team.name.clone(),
            shots: 0,
            goals: 0,
            total_xg: 0.0,
            passes_attempted: 0,
            passes_completed: 0,
        });

        match ev.event_type.name.as_str() {
            "Shot" => {
                entry.shots += 1;
                if let Some(shot) = &ev.shot {
                    entry.total_xg += shot.statsbomb_xg.unwrap_or(0.0);
                    if shot.outcome.as_ref().is_some_and(|o| o.name == "Goal") {
                        entry.goals += 1;
                    }
                }
            }
            "Pass" => {
                entry.passes_attempted += 1;
                // In StatsBomb, a Pass with outcome == None was completed
                let completed = ev.pass.as_ref().is_some_and(|p| p.outcome.is_none());
                if completed {
                    entry.passes_completed += 1;
                }
            }
            _ => {}
        }
    }

    let mut summaries: Vec<PlayerMatchSummary> = map
        .into_iter()
        .map(|(player_id, a)| {
            let pass_accuracy_pct = if a.passes_attempted > 0 {
                ((a.passes_completed as f64 / a.passes_attempted as f64) * 1000.0).round() / 10.0
            } else {
                0.0
            };
            PlayerMatchSummary {
                player_id,
                player_name: a.name,
                team_name: a.team,
                shots: a.shots,
                goals: a.goals,
                total_xg: (a.total_xg * 1000.0).round() / 1000.0,
                passes_attempted: a.passes_attempted,
                passes_completed: a.passes_completed,
                pass_accuracy_pct,
            }
        })
        .collect();

    summaries.sort_by(|a, b| {
        b.total_xg
            .partial_cmp(&a.total_xg)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.passes_completed.cmp(&a.passes_completed))
    });

    summaries
}

/// Idempotently persist teams, players, and match events into PostgreSQL.
pub async fn persist_events_to_db(
    pool: &PgPool,
    match_id: i32,
    events: &[StatsBombEvent],
) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;

    // Collect unique teams and players first
    let mut teams: HashMap<i32, &str> = HashMap::new();
    let mut players: HashMap<i32, (&str, Option<&str>)> = HashMap::new();

    for ev in events {
        teams.entry(ev.team.id).or_insert(&ev.team.name);
        if let Some(p) = &ev.player {
            let pos = ev.position.as_ref().map(|pos| pos.name.as_str());
            players.entry(p.id).or_insert((&p.name, pos));
        }
    }

    for (id, name) in &teams {
        sqlx::query("INSERT INTO teams (id, name) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING")
            .bind(id)
            .bind(name)
            .execute(&mut *tx)
            .await?;
    }

    for (id, (name, pos)) in &players {
        sqlx::query(
            "INSERT INTO players (id, name, primary_position) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(name)
        .bind(pos)
        .execute(&mut *tx)
        .await?;
    }

    let mut inserted = 0usize;
    for ev in events {
        let loc_x = ev.location.as_ref().and_then(|l| l.first().copied());
        let loc_y = ev.location.as_ref().and_then(|l| l.get(1).copied());
        let (end_x, end_y, xg, outcome, extra_val) = if let Some(shot) = &ev.shot {
            (
                shot.end_location.as_ref().and_then(|l| l.first().copied()),
                shot.end_location.as_ref().and_then(|l| l.get(1).copied()),
                shot.statsbomb_xg,
                shot.outcome.as_ref().map(|o| o.name.as_str()),
                serde_json::to_value(shot).ok(),
            )
        } else if let Some(pass) = &ev.pass {
            (
                pass.end_location.as_ref().and_then(|l| l.first().copied()),
                pass.end_location.as_ref().and_then(|l| l.get(1).copied()),
                None,
                pass.outcome.as_ref().map(|o| o.name.as_str()),
                serde_json::to_value(pass).ok(),
            )
        } else if let Some(carry) = &ev.carry {
            (
                carry.end_location.as_ref().and_then(|l| l.first().copied()),
                carry.end_location.as_ref().and_then(|l| l.get(1).copied()),
                None,
                None,
                None,
            )
        } else {
            (None, None, None, None, None)
        };

        let res = sqlx::query(
            r#"
            INSERT INTO match_events (
                id, match_id, event_index, period, minute, second, possession,
                team_id, player_id, event_type, outcome,
                location_x, location_y, end_x, end_y, xg, extra_data
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17
            ) ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(ev.id)
        .bind(match_id)
        .bind(ev.index)
        .bind(ev.period)
        .bind(ev.minute)
        .bind(ev.second)
        .bind(ev.possession)
        .bind(ev.team.id)
        .bind(ev.player.as_ref().map(|p| p.id))
        .bind(&ev.event_type.name)
        .bind(outcome)
        .bind(loc_x)
        .bind(loc_y)
        .bind(end_x)
        .bind(end_y)
        .bind(xg)
        .bind(extra_val)
        .execute(&mut *tx)
        .await?;

        inserted += res.rows_affected() as usize;
    }

    tx.commit().await?;
    Ok(inserted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_players_xg_and_pass_accuracy() {
        let raw_json = r#"[
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "index": 1, "period": 1, "minute": 10, "second": 5, "possession": 3,
                "type": {"id": 16, "name": "Shot"},
                "team": {"id": 779, "name": "Argentina"},
                "player": {"id": 5503, "name": "Lionel Andrés Messi Cuccittini"},
                "location": [108.0, 40.0],
                "shot": {"statsbomb_xg": 0.7835, "outcome": {"id": 97, "name": "Goal"}}
            },
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "index": 2, "period": 1, "minute": 12, "second": 10, "possession": 4,
                "type": {"id": 30, "name": "Pass"},
                "team": {"id": 779, "name": "Argentina"},
                "player": {"id": 5503, "name": "Lionel Andrés Messi Cuccittini"},
                "location": [75.0, 42.0],
                "pass": {"end_location": [95.0, 30.0]}
            },
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "index": 3, "period": 1, "minute": 14, "second": 20, "possession": 5,
                "type": {"id": 30, "name": "Pass"},
                "team": {"id": 779, "name": "Argentina"},
                "player": {"id": 5503, "name": "Lionel Andrés Messi Cuccittini"},
                "location": [80.0, 35.0],
                "pass": {"end_location": [102.0, 20.0], "outcome": {"id": 9, "name": "Incomplete"}}
            }
        ]"#;

        let events: Vec<StatsBombEvent> = serde_json::from_str(raw_json).unwrap();
        let summary = summarize_players(&events);

        assert_eq!(summary.len(), 1);
        let messi = &summary[0];
        assert_eq!(messi.shots, 1);
        assert_eq!(messi.goals, 1);
        assert!((messi.total_xg - 0.784).abs() < 0.001);
        assert_eq!(messi.passes_attempted, 2);
        assert_eq!(messi.passes_completed, 1);
        assert_eq!(messi.pass_accuracy_pct, 50.0);
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct EspnSummaryRaw {
    pub header: Option<EspnHeader>,
    pub boxscore: Option<EspnBoxscore>,
    pub rosters: Option<Vec<EspnRoster>>,
}

#[derive(Debug, Deserialize)]
pub struct EspnHeader {
    pub competitions: Option<Vec<EspnCompetition>>,
}

#[derive(Debug, Deserialize)]
pub struct EspnCompetition {
    pub date: Option<String>,
    pub competitors: Vec<EspnCompetitor>,
}

#[derive(Debug, Deserialize)]
pub struct EspnCompetitor {
    #[serde(rename = "homeAway")]
    pub home_away: String,
    pub score: Option<String>,
    pub team: EspnTeamName,
}

#[derive(Debug, Deserialize)]
pub struct EspnTeamName {
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct EspnBoxscore {
    pub teams: Option<Vec<EspnBoxTeam>>,
}

#[derive(Debug, Deserialize)]
pub struct EspnBoxTeam {
    pub team: EspnTeamName,
    pub statistics: Option<Vec<EspnStatItem>>,
}

#[derive(Debug, Deserialize)]
pub struct EspnStatItem {
    pub name: String,
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct EspnRoster {
    pub team: EspnTeamName,
    pub roster: Option<Vec<EspnPlayerEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct EspnPlayerEntry {
    pub athlete: EspnTeamName,
    pub stats: Option<Vec<EspnStatItem>>,
}

#[derive(Debug, Serialize)]
pub struct LiveTeamMetrics {
    pub team_name: String,
    pub home_away: String,
    pub score: String,
    pub stats: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct LivePlayerImpact {
    pub player_name: String,
    pub team_name: String,
    pub shots: u32,
    pub shots_on_target: u32,
    pub goals: u32,
    pub assists: u32,
}

#[derive(Debug, Serialize)]
pub struct LiveMatchReport {
    pub league: String,
    pub event_id: String,
    pub match_date: String,
    pub teams: Vec<LiveTeamMetrics>,
    pub top_players: Vec<LivePlayerImpact>,
}

/// Fetch and structure a live or recent 2026 match summary via ESPN's public feed.
pub async fn fetch_live_match_report(
    league: &str,
    event_id: &str,
) -> anyhow::Result<LiveMatchReport> {
    let url = format!(
        "https://site.api.espn.com/apis/site/v2/sports/soccer/{league}/summary?event={event_id}"
    );
    let client = reqwest::Client::new();
    let raw = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<EspnSummaryRaw>()
        .await?;

    Ok(transform_espn_summary(league, event_id, raw))
}

pub fn transform_espn_summary(
    league: &str,
    event_id: &str,
    raw: EspnSummaryRaw,
) -> LiveMatchReport {
    let mut match_date = String::new();
    let mut home_away_map: HashMap<String, (String, String)> = HashMap::new();

    if let Some(header) = &raw.header {
        if let Some(comps) = &header.competitions {
            if let Some(comp) = comps.first() {
                match_date = comp.date.clone().unwrap_or_default();
                for c in &comp.competitors {
                    home_away_map.insert(
                        c.team.display_name.clone(),
                        (
                            c.home_away.clone(),
                            c.score.clone().unwrap_or_else(|| "0".to_string()),
                        ),
                    );
                }
            }
        }
    }

    let mut teams = Vec::new();
    if let Some(boxscore) = raw.boxscore {
        if let Some(box_teams) = boxscore.teams {
            for bt in box_teams {
                let name = bt.team.display_name;
                let (home_away, score) = home_away_map
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| ("unknown".to_string(), "0".to_string()));
                let mut stats = HashMap::new();
                for st in bt.statistics.unwrap_or_default() {
                    if let Some(val) = st.display_value {
                        stats.insert(st.name, val);
                    }
                }
                teams.push(LiveTeamMetrics {
                    team_name: name,
                    home_away,
                    score,
                    stats,
                });
            }
        }
    }

    let mut top_players = Vec::new();
    for r in raw.rosters.unwrap_or_default() {
        let team_name = r.team.display_name;
        for p in r.roster.unwrap_or_default() {
            let mut shots = 0u32;
            let mut sot = 0u32;
            let mut goals = 0u32;
            let mut assists = 0u32;

            for st in p.stats.unwrap_or_default() {
                let v = st.value.unwrap_or(0.0) as u32;
                match st.name.as_str() {
                    "totalShots" => shots = v,
                    "shotsOnTarget" => sot = v,
                    "totalGoals" => goals = v,
                    "goalAssists" => assists = v,
                    _ => {}
                }
            }

            if shots > 0 || goals > 0 || assists > 0 {
                top_players.push(LivePlayerImpact {
                    player_name: p.athlete.display_name,
                    team_name: team_name.clone(),
                    shots,
                    shots_on_target: sot,
                    goals,
                    assists,
                });
            }
        }
    }

    top_players.sort_by(|a, b| {
        (b.goals * 3 + b.assists * 2)
            .cmp(&(a.goals * 3 + a.assists * 2))
            .then_with(|| b.shots.cmp(&a.shots))
    });

    LiveMatchReport {
        league: league.to_string(),
        event_id: event_id.to_string(),
        match_date,
        teams,
        top_players,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_espn_summary() {
        let raw_json = r#"{
            "header": {
                "competitions": [{
                    "date": "2026-11-24T20:00Z",
                    "competitors": [
                        {"homeAway": "home", "score": "5", "team": {"displayName": "Real Madrid"}},
                        {"homeAway": "away", "score": "1", "team": {"displayName": "PSV Eindhoven"}}
                    ]
                }]
            },
            "boxscore": {
                "teams": [
                    {"team": {"displayName": "Real Madrid"}, "statistics": [{"name": "possessionPct", "displayValue": "61.5"}]}
                ]
            },
            "rosters": [
                {
                    "team": {"displayName": "Real Madrid"},
                    "roster": [
                        {
                            "athlete": {"displayName": "Kylian Mbappé"},
                            "stats": [
                                {"name": "totalShots", "value": 9.0},
                                {"name": "shotsOnTarget", "value": 3.0},
                                {"name": "totalGoals", "value": 2.0},
                                {"name": "goalAssists", "value": 1.0}
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let parsed: EspnSummaryRaw = serde_json::from_str(raw_json).unwrap();
        let report = transform_espn_summary("uefa.champions", "401915298", parsed);

        assert_eq!(report.teams.len(), 1);
        assert_eq!(report.teams[0].score, "5");
        assert_eq!(report.top_players.len(), 1);
        assert_eq!(report.top_players[0].goals, 2);
        assert_eq!(report.top_players[0].assists, 1);
    }
}

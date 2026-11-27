use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Strongly-typed StatsBomb Open Data event JSON representation.
#[derive(Debug, Deserialize)]
pub struct StatsBombEvent {
    pub id: Uuid,
    pub index: i32,
    pub period: i32,
    pub minute: i32,
    pub second: i32,
    pub possession: i32,
    #[serde(rename = "type")]
    pub event_type: NamedEntity,
    pub team: NamedEntity,
    pub player: Option<NamedEntity>,
    pub position: Option<NamedEntity>,
    pub location: Option<Vec<f64>>,
    pub pass: Option<StatsBombPass>,
    pub shot: Option<StatsBombShot>,
    pub carry: Option<StatsBombCarry>,
}

#[derive(Debug, Deserialize)]
pub struct NamedEntity {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatsBombPass {
    pub end_location: Option<Vec<f64>>,
    pub outcome: Option<NamedEntityOut>,
    pub length: Option<f64>,
    pub angle: Option<f64>,
    pub height: Option<NamedEntityOut>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatsBombShot {
    pub end_location: Option<Vec<f64>>,
    pub statsbomb_xg: Option<f64>,
    pub outcome: Option<NamedEntityOut>,
    pub body_part: Option<NamedEntityOut>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatsBombCarry {
    pub end_location: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamedEntityOut {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub engine: &'static str,
    pub db_connected: bool,
}

#[derive(Debug, Serialize)]
pub struct PlayerMatchSummary {
    pub player_id: i32,
    pub player_name: String,
    pub team_name: String,
    pub shots: u32,
    pub goals: u32,
    pub total_xg: f64,
    pub passes_attempted: u32,
    pub passes_completed: u32,
    pub pass_accuracy_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct MatchSummaryResponse {
    pub match_id: i32,
    pub total_events: usize,
    pub players: Vec<PlayerMatchSummary>,
}

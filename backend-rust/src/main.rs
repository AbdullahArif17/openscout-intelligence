mod ingest;
mod live;
mod models;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use live::LiveMatchReport;
use models::{HealthResponse, MatchSummaryResponse};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route(
            "/api/v1/matches/:match_id/players/summary",
            get(get_match_player_summary),
        )
        .route(
            "/api/v1/matches/:match_id/ingest",
            post(ingest_match_events),
        )
        .route(
            "/api/v1/live/:league/matches/:event_id",
            get(get_live_match_report),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn get_live_match_report(
    Path((league, event_id)): Path<(String, String)>,
) -> Result<Json<LiveMatchReport>, (StatusCode, String)> {
    let report = live::fetch_live_match_report(&league, &event_id)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Live feed fetch failed: {e}")))?;
    Ok(Json(report))
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        engine: "openscout-core-rust",
        db_connected: state.db.is_some(),
    })
}

async fn get_match_player_summary(
    Path(match_id): Path<i32>,
) -> Result<Json<MatchSummaryResponse>, (StatusCode, String)> {
    let events = ingest::fetch_statsbomb_events(match_id)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("StatsBomb fetch failed: {e}")))?;

    let players = ingest::summarize_players(&events);
    Ok(Json(MatchSummaryResponse {
        match_id,
        total_events: events.len(),
        players,
    }))
}

async fn ingest_match_events(
    State(state): State<AppState>,
    Path(match_id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(pool) = &state.db else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "PostgreSQL is not connected. Start Docker and set DATABASE_URL.".to_string(),
        ));
    };

    let events = ingest::fetch_statsbomb_events(match_id)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("StatsBomb fetch failed: {e}")))?;

    let inserted = ingest::persist_events_to_db(pool, match_id, &events)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB insert failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "match_id": match_id,
        "fetched_events": events.len(),
        "inserted_rows": inserted
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "openscout_core=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = match std::env::var("DATABASE_URL") {
        Ok(url) => match PgPoolOptions::new().max_connections(5).connect(&url).await {
            Ok(pool) => {
                sqlx::migrate!("./migrations").run(&pool).await?;
                tracing::info!("Connected to PostgreSQL and applied migrations");
                Some(pool)
            }
            Err(err) => {
                tracing::warn!("PostgreSQL unavailable ({err}); running in stateless API mode");
                None
            }
        },
        Err(_) => {
            tracing::warn!("DATABASE_URL not set; running in stateless API mode");
            None
        }
    };

    let app = build_router(AppState { db });
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("OpenScout Rust Core listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = build_router(AppState { db: None });
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("openscout-core-rust"));
    }
}

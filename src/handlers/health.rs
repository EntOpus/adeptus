use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub components: HealthComponents,
}

#[derive(Serialize)]
pub struct HealthComponents {
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nats: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pactum: Option<String>,
}

pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let db_status = match state.db.health_check().await {
        Ok(()) => "healthy".to_string(),
        Err(e) => {
            tracing::warn!("Database health check failed: {}", e);
            "unhealthy".to_string()
        }
    };

    let nats_status = if state.events.is_configured() {
        Some(if state.events.is_connected() {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        })
    } else {
        None
    };

    let pactum_status = if state.pactum.is_configured() {
        Some("configured".to_string())
    } else {
        None
    };

    let overall_status =
        if db_status == "healthy" && nats_status.as_deref().unwrap_or("healthy") == "healthy" {
            "healthy"
        } else {
            "degraded"
        };

    Ok(Json(HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        components: HealthComponents {
            database: db_status,
            nats: nats_status,
            pactum: pactum_status,
        },
    }))
}

pub async fn readiness_check(State(state): State<AppState>) -> StatusCode {
    match state.db.health_check().await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub async fn liveness_check() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liveness_check() {
        let status = liveness_check().await;
        assert_eq!(status, StatusCode::OK);
    }
}

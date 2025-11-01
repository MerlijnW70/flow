use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
}

/// Health check handler
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy")
    )
)]
async fn health_check(State(pool): State<PgPool>) -> impl IntoResponse {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Calculate uptime (simplified - would use actual start time in production)
    let uptime = 0; // Placeholder

    let response = HealthResponse {
        status: if db_status == "healthy" {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        database: db_status.to_string(),
        uptime_seconds: uptime,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let status_code = if db_status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

/// Liveness probe (Kubernetes-style)
async fn liveness() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

/// Readiness probe
async fn readiness(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => (StatusCode::OK, "ready"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "not ready"),
    }
}

pub fn routes(db_pool: PgPool) -> Router {
    Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/health/live", get(liveness))
        .route("/api/v1/health/ready", get(readiness))
        .with_state(db_pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_liveness_probe() {
        let app = Router::new().route("/live", get(liveness));

        let response = app
            .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

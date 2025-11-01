use axum::{routing::get, Router, Json};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::Serialize;
use std::sync::Arc;

use crate::utils::error::AppResult;
use crate::utils::response::ApiResponse;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct ReadinessResponse {
    ready: bool,
    checks: Vec<CheckResult>,
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

pub fn init_metrics() -> PrometheusHandle {
    START_TIME.set(std::time::Instant::now()).ok();

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
        .unwrap()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
}

async fn health_handler() -> impl axum::response::IntoResponse {
    let uptime = START_TIME
        .get()
        .map(|start| start.elapsed().as_secs())
        .unwrap_or(0);

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    };

    ApiResponse::success(response)
}

async fn readiness_handler() -> impl axum::response::IntoResponse {
    // In production, you would check:
    // - Database connectivity
    // - External service availability
    // - Cache connectivity
    // etc.

    let checks = vec![
        CheckResult {
            name: "api".to_string(),
            healthy: true,
            message: None,
        },
        // Add more checks as needed
    ];

    let ready = checks.iter().all(|check| check.healthy);

    let response = ReadinessResponse { ready, checks };

    ApiResponse::success(response)
}

async fn metrics_handler() -> String {
    // This would return Prometheus-formatted metrics
    // In production, you'd use the PrometheusHandle to export metrics
    "# Metrics endpoint\n".to_string()
}

// Utility functions for recording metrics
pub fn record_request(method: &str, path: &str, status: u16, duration: f64) {
    let method = method.to_string();
    let path = path.to_string();
    let status = status.to_string();
    metrics::counter!("http_requests_total", "method" => method.clone(), "path" => path.clone(), "status" => status).increment(1);
    metrics::histogram!("http_requests_duration_seconds", "method" => method, "path" => path).record(duration);
}

pub fn record_database_query(query_type: &str, duration: f64) {
    let query_type = query_type.to_string();
    metrics::histogram!("database_query_duration_seconds", "type" => query_type).record(duration);
}

pub fn record_external_api_call(provider: &str, success: bool, duration: f64) {
    let provider = provider.to_string();
    let status = if success { "success" } else { "failure" }.to_string();
    metrics::counter!("external_api_calls_total", "provider" => provider.clone(), "status" => status).increment(1);
    metrics::histogram!("external_api_duration_seconds", "provider" => provider).record(duration);
}

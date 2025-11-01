use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VersionResponse {
    pub version: String,
    pub commit_hash: String,
    pub build_timestamp: String,
    pub uptime_seconds: u64,
    pub rust_version: String,
}

/// Version information handler
#[utoipa::path(
    get,
    path = "/api/v1/version",
    tag = "version",
    responses(
        (status = 200, description = "Version information", body = VersionResponse)
    )
)]
async fn version_info() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    let commit_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    let build_timestamp = option_env!("BUILD_TIMESTAMP").unwrap_or("unknown");
    let rust_version = env!("CARGO_PKG_RUST_VERSION");

    // Calculate uptime (simplified - would use actual start time in production)
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 86400; // Placeholder: reset daily

    let response = VersionResponse {
        version: version.to_string(),
        commit_hash: commit_hash.to_string(),
        build_timestamp: build_timestamp.to_string(),
        uptime_seconds: uptime,
        rust_version: rust_version.to_string(),
    };

    (StatusCode::OK, Json(response))
}

pub fn routes() -> Router {
    Router::new().route("/api/v1/version", get(version_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_version_endpoint() {
        let app = routes();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/version")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["version"].is_string());
        assert!(json["commit_hash"].is_string());
        assert!(json["rust_version"].is_string());
    }
}

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConfigResponse {
    pub environment: String,
    pub features: Vec<String>,
    pub max_upload_size_mb: u64,
    pub rate_limit_per_minute: u32,
    pub cors_origins: Vec<String>,
}

/// Config information handler (safe runtime config)
#[utoipa::path(
    get,
    path = "/api/v1/config",
    tag = "config",
    responses(
        (status = 200, description = "Runtime configuration", body = ConfigResponse)
    )
)]
async fn config_info() -> impl IntoResponse {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let mut features = vec!["auth".to_string(), "users".to_string()];

    #[cfg(feature = "ai")]
    features.push("ai".to_string());

    #[cfg(feature = "websocket")]
    features.push("websocket".to_string());

    #[cfg(feature = "jobs")]
    features.push("jobs".to_string());

    #[cfg(feature = "storage")]
    features.push("storage".to_string());

    let cors_origins = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.to_string())
        .collect();

    let response = ConfigResponse {
        environment,
        features,
        max_upload_size_mb: 10,
        rate_limit_per_minute: 60,
        cors_origins,
    };

    (StatusCode::OK, Json(response))
}

pub fn routes() -> Router {
    Router::new().route("/api/v1/config", get(config_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_config_endpoint() {
        let app = routes();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["environment"].is_string());
        assert!(json["features"].is_array());
        assert!(json["features"].as_array().unwrap().len() >= 2);
    }
}

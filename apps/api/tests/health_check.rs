// Health Check Integration Tests
// Validates that health endpoints respond correctly

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use std::time::Instant;

use common::create_minimal_test_app;

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    // Arrange
    let app = create_minimal_test_app();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_returns_json() {
    // Arrange
    let app = create_minimal_test_app();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_health_endpoint_response_time() {
    // Arrange
    let app = create_minimal_test_app();

    // Act
    let start = Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let duration = start.elapsed();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        duration.as_millis() < 100,
        "Health check took {}ms, expected < 100ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_health_endpoint_multiple_requests() {
    // Arrange
    let app = create_minimal_test_app();

    // Act & Assert - Test 5 consecutive requests
    for i in 0..5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} failed",
            i + 1
        );
    }
}

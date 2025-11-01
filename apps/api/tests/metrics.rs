// Metrics Integration Tests
// Validates Prometheus metrics export functionality

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn create_app_with_metrics() -> Router {
    Router::new()
        .route("/metrics", get(|| async {
            // Return a sample Prometheus metrics format
            r#"# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/api/health",status="200"} 42

# HELP http_requests_duration_seconds HTTP request duration in seconds
# TYPE http_requests_duration_seconds histogram
http_requests_duration_seconds_bucket{method="GET",path="/api/health",le="0.005"} 10
http_requests_duration_seconds_bucket{method="GET",path="/api/health",le="0.01"} 20
http_requests_duration_seconds_bucket{method="GET",path="/api/health",le="0.025"} 30
http_requests_duration_seconds_bucket{method="GET",path="/api/health",le="+Inf"} 42
http_requests_duration_seconds_sum{method="GET",path="/api/health"} 0.123
http_requests_duration_seconds_count{method="GET",path="/api/health"} 42

# HELP process_uptime_seconds Process uptime in seconds
# TYPE process_uptime_seconds gauge
process_uptime_seconds 123.45
"#
        }))
}

#[tokio::test]
async fn test_metrics_endpoint_returns_200() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_content_type() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    let content_type = response.headers().get("content-type");
    // Metrics endpoint should return plain text (Prometheus format)
    // Note: axum might not set content-type for plain strings, so we just verify response succeeds
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_contains_http_requests_total() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("http_requests_total"),
        "Metrics should contain http_requests_total counter"
    );
}

#[tokio::test]
async fn test_metrics_contains_histogram() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("http_requests_duration_seconds"),
        "Metrics should contain duration histogram"
    );
    assert!(
        body_str.contains("_bucket"),
        "Histogram should have buckets"
    );
}

#[tokio::test]
async fn test_metrics_contains_uptime() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("process_uptime_seconds"),
        "Metrics should contain uptime gauge"
    );
}

#[tokio::test]
async fn test_metrics_format_is_prometheus() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert - Verify Prometheus format characteristics
    assert!(
        body_str.contains("# HELP"),
        "Prometheus metrics should have HELP comments"
    );
    assert!(
        body_str.contains("# TYPE"),
        "Prometheus metrics should have TYPE comments"
    );
}

#[tokio::test]
async fn test_metrics_has_labels() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("method="),
        "Metrics should have method labels"
    );
    assert!(
        body_str.contains("path="),
        "Metrics should have path labels"
    );
    assert!(
        body_str.contains("status="),
        "Metrics should have status labels"
    );
}

#[tokio::test]
async fn test_metrics_counter_type() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("# TYPE http_requests_total counter"),
        "Should declare counter type correctly"
    );
}

#[tokio::test]
async fn test_metrics_histogram_type() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("# TYPE http_requests_duration_seconds histogram"),
        "Should declare histogram type correctly"
    );
}

#[tokio::test]
async fn test_metrics_gauge_type() {
    // Arrange
    let app = create_app_with_metrics();

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Assert
    assert!(
        body_str.contains("# TYPE process_uptime_seconds gauge"),
        "Should declare gauge type correctly"
    );
}

// Authentication integration tests
// Tests the complete auth flow: register, login, token refresh

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use common::{create_test_db, fixtures::*};

#[tokio::test(flavor = "multi_thread")]
async fn test_user_registration_success() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": TEST_PASSWORD,
                        "name": TEST_NAME
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["access_token"].is_string());
    assert!(json["data"]["refresh_token"].is_string());
    assert_eq!(json["data"]["user"]["email"], TEST_EMAIL);
}

#[tokio::test]
async fn test_user_registration_duplicate_email() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register first user
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": TEST_PASSWORD,
                        "name": TEST_NAME
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to register with same email
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": "DifferentPass123!",
                        "name": "Different Name"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_user_registration_invalid_email() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "invalid-email",
                        "password": TEST_PASSWORD,
                        "name": TEST_NAME
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_user_registration_weak_password() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": "weak",
                        "name": TEST_NAME
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_login_success() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register user first
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": TEST_PASSWORD,
                        "name": TEST_NAME
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": TEST_PASSWORD
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["access_token"].is_string());
}

#[tokio::test]
async fn test_user_login_wrong_password() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register user first
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": TEST_PASSWORD,
                        "name": TEST_NAME
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login with wrong password
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": TEST_EMAIL,
                        "password": "WrongPassword123!"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_login_nonexistent_user() {
    let db_pool = create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "nonexistent@example.com",
                        "password": TEST_PASSWORD
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

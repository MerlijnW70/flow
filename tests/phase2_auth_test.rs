mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use common::fixtures::{TEST_EMAIL, TEST_PASSWORD, TEST_NAME};

#[tokio::test(flavor = "multi_thread")]
async fn test_signup_success() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
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
    assert_eq!(json["data"]["user"]["role"], "user");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_signup_with_admin_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "admin@example.com",
                        "password": TEST_PASSWORD,
                        "name": "Admin User",
                        "role": "admin"
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
    assert_eq!(json["data"]["user"]["role"], "admin");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_signup_duplicate() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // First signup
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Duplicate signup
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_login_success() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register first
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME
                }).to_string()))
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
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["access_token"].is_string());
    assert!(json["data"]["refresh_token"].is_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_login_invalid_password() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register first
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME
                }).to_string()))
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
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": "WrongPassword123!"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_role_guard_admin_required() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register as regular user
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Try to access admin-only endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_admin_can_list_users() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register as admin
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": "admin@example.com",
                    "password": TEST_PASSWORD,
                    "name": "Admin User",
                    "role": "admin"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Access admin-only endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_moderator_cannot_list_users() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register as moderator
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": "moderator@example.com",
                    "password": TEST_PASSWORD,
                    "name": "Moderator User",
                    "role": "moderator"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Try to access admin-only endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_jwt_token_contains_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register with specific role
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME,
                    "role": "moderator"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response includes role
    assert_eq!(json["data"]["user"]["role"], "moderator");

    // Verify token is present
    assert!(json["data"]["access_token"].is_string());
    assert!(json["data"]["refresh_token"].is_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_can_access_own_profile() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Register
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": TEST_EMAIL,
                    "password": TEST_PASSWORD,
                    "name": TEST_NAME
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Access own profile
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["email"], TEST_EMAIL);
    assert_eq!(json["data"]["role"], "user");
}

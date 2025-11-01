mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use common::fixtures::*;

// ============================================================================
// REGISTRATION TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_success_default_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("user");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
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
    assert_eq!(json["data"]["token_type"], "Bearer");
    assert_eq!(json["data"]["user"]["role"], "user");
    assert_eq!(json["data"]["user"]["email"], email);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_with_admin_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("admin");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["user"]["role"], "admin");
    assert_eq!(json["data"]["user"]["email"], email);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_with_moderator_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("moderator");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, MODERATOR_PASSWORD, MODERATOR_NAME, "moderator")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["user"]["role"], "moderator");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_duplicate_email() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("duplicate");

    // First registration
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Duplicate registration
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_invalid_email() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload("not-an-email", TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_weak_password() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("weak");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, "short", TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_registration_empty_name() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("noname");
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, "").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// LOGIN TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn test_login_success_with_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("login");

    // Register first
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, TEST_PASSWORD, TEST_NAME, "admin").to_string(),
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
                .body(Body::from(login_payload(&email, TEST_PASSWORD).to_string()))
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
    assert_eq!(json["data"]["user"]["role"], "admin");
    assert_eq!(json["data"]["user"]["email"], email);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_login_invalid_email() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    login_payload("nonexistent@test.com", TEST_PASSWORD).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_login_invalid_password() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("wrongpass");

    // Register first
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
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
                .body(Body::from(login_payload(&email, "WrongPassword123!").to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// ROLE-BASED ACCESS CONTROL TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn test_user_cannot_access_admin_endpoints() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("regularuser");

    // Register as regular user
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Try to access admin-only endpoint (list users)
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
async fn test_admin_can_access_admin_endpoints() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("adminuser");

    // Register as admin
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Access admin-only endpoint (list users)
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
async fn test_moderator_cannot_access_admin_only_endpoints() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("moduser");

    // Register as moderator
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, MODERATOR_PASSWORD, MODERATOR_NAME, "moderator")
                        .to_string(),
                ))
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
async fn test_all_users_can_access_own_profile() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Test for each role: user, admin, moderator
    for (email_prefix, role) in [("user", "user"), ("admin", "admin"), ("moderator", "moderator")] {
        let email = unique_email(email_prefix);

        let signup_response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/signup")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        user_registration_with_role(&email, TEST_PASSWORD, TEST_NAME, role)
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = json["data"]["access_token"].as_str().unwrap();

        // Access own profile
        let response = app.clone()
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
        let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(profile["data"]["email"], email);
        assert_eq!(profile["data"]["role"], role);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_unauthenticated_cannot_access_protected_routes() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Try to access protected endpoint without token
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_invalid_token_rejected() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/me")
                .header("authorization", "Bearer invalid_token_here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// JWT TOKEN TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn test_jwt_contains_correct_role_claims() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("jwttest");

    // Register with admin role
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response includes role
    assert_eq!(json["data"]["user"]["role"], "admin");
    assert_eq!(json["data"]["user"]["email"], email);

    // Tokens should be present
    assert!(json["data"]["access_token"].is_string());
    assert!(json["data"]["refresh_token"].is_string());
    assert_eq!(json["data"]["token_type"], "Bearer");
    assert!(json["data"]["expires_in"].is_number());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_refresh_preserves_role() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("refresh");

    // Register
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let refresh_token = json["data"]["refresh_token"].as_str().unwrap();

    // Refresh token
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"refresh_token": refresh_token}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let refreshed: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify role is preserved
    assert_eq!(refreshed["data"]["user"]["role"], "admin");
    assert!(refreshed["data"]["access_token"].is_string());
}

// ============================================================================
// USER PROFILE TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn test_user_can_update_own_profile() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("updateprofile");

    // Register
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Update profile
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(update_user_payload("Updated Name").to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated["data"]["name"], "Updated Name");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_can_change_password() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("changepass");
    let new_password = "NewSecurePassword123!";

    // Register
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Change password
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/users/me/password")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    change_password_payload(TEST_PASSWORD, new_password).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify can login with new password
    let login_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_payload(&email, new_password).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_can_delete_own_account() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("deleteaccount");

    // Register
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&email, TEST_PASSWORD, TEST_NAME).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["data"]["access_token"].as_str().unwrap();

    // Delete account
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

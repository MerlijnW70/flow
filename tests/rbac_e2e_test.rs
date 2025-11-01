mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use common::fixtures::*;

// ============================================================================
// END-TO-END ROLE-BASED ACCESS CONTROL TESTS
// ============================================================================

/// Test complete workflow: User signup -> Cannot access admin routes
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_user_role_restrictions() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let email = unique_email("e2e_user");

    // Step 1: User signs up (default role: user)
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

    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let signup_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = signup_json["data"]["access_token"].as_str().unwrap();

    // Step 2: User can access their own profile
    let profile_response = app.clone()
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

    assert_eq!(profile_response.status(), StatusCode::OK);

    // Step 3: User CANNOT list all users (admin-only)
    let list_response = app.clone()
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

    assert_eq!(list_response.status(), StatusCode::FORBIDDEN);

    // Step 4: User can update their own profile
    let update_response = app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(update_user_payload("New Name").to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);

    // Step 5: User can change their password
    let password_response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/users/me/password")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    change_password_payload(TEST_PASSWORD, "NewPassword123!").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(password_response.status(), StatusCode::OK);
}

/// Test complete workflow: Admin signup -> Full access to all routes
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_admin_role_full_access() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let admin_email = unique_email("e2e_admin");
    let user_email = unique_email("e2e_regular_user");

    // Step 1: Create a regular user first
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&user_email, TEST_PASSWORD, "Regular User").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Step 2: Admin signs up
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&admin_email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let signup_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_token = signup_json["data"]["access_token"].as_str().unwrap();
    assert_eq!(signup_json["data"]["user"]["role"], "admin");

    // Step 3: Admin can access their own profile
    let profile_response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(profile_response.status(), StatusCode::OK);

    // Step 4: Admin CAN list all users
    let list_response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have at least 2 users (regular user + admin)
    assert!(list_json["data"].is_array());
    assert!(list_json["data"].as_array().unwrap().len() >= 2);

    // Step 5: Admin can update their profile
    let update_response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", admin_token))
                .header("content-type", "application/json")
                .body(Body::from(update_user_payload("Admin Updated").to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
}

/// Test complete workflow: Moderator signup -> Limited admin access
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_moderator_role_limited_access() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let mod_email = unique_email("e2e_moderator");

    // Step 1: Moderator signs up
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&mod_email, MODERATOR_PASSWORD, MODERATOR_NAME, "moderator")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let signup_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let mod_token = signup_json["data"]["access_token"].as_str().unwrap();
    assert_eq!(signup_json["data"]["user"]["role"], "moderator");

    // Step 2: Moderator can access their own profile
    let profile_response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users/me")
                .header("authorization", format!("Bearer {}", mod_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(profile_response.status(), StatusCode::OK);

    // Step 3: Moderator CANNOT list users (admin-only in current implementation)
    let list_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", mod_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::FORBIDDEN);
}

/// Test JWT refresh preserves role across token refresh
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_role_preserved_across_refresh() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let admin_email = unique_email("e2e_refresh_admin");

    // Step 1: Admin signs up
    let signup_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&admin_email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = hyper::body::to_bytes(signup_response.into_body()).await.unwrap();
    let signup_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let refresh_token = signup_json["data"]["refresh_token"].as_str().unwrap();

    // Step 2: Use refresh token to get new access token
    let refresh_response = app.clone()
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

    assert_eq!(refresh_response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(refresh_response.into_body()).await.unwrap();
    let refresh_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Step 3: Verify role is still admin
    assert_eq!(refresh_json["data"]["user"]["role"], "admin");
    let new_token = refresh_json["data"]["access_token"].as_str().unwrap();

    // Step 4: Verify new token still has admin access
    let list_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", new_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
}

/// Test login preserves role from registration
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_role_preserved_across_login() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    let admin_email = unique_email("e2e_login_admin");

    // Step 1: Register as admin
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&admin_email, ADMIN_PASSWORD, ADMIN_NAME, "admin")
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Step 2: Login
    let login_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_payload(&admin_email, ADMIN_PASSWORD).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(login_response.into_body()).await.unwrap();
    let login_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Step 3: Verify role is still admin
    assert_eq!(login_json["data"]["user"]["role"], "admin");
    let token = login_json["data"]["access_token"].as_str().unwrap();

    // Step 4: Verify admin access still works
    let list_response = app
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

    assert_eq!(list_response.status(), StatusCode::OK);
}

/// Test multiple users with different roles can coexist
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_multiple_users_different_roles() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Create users with different roles
    let user_email = unique_email("multi_user");
    let admin_email = unique_email("multi_admin");
    let mod_email = unique_email("multi_mod");

    // Register user
    let user_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_payload(&user_email, TEST_PASSWORD, "User").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let user_body = hyper::body::to_bytes(user_response.into_body()).await.unwrap();
    let user_json: serde_json::Value = serde_json::from_slice(&user_body).unwrap();
    let user_token = user_json["data"]["access_token"].as_str().unwrap().to_string();

    // Register admin
    let admin_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&admin_email, ADMIN_PASSWORD, "Admin", "admin").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let admin_body = hyper::body::to_bytes(admin_response.into_body()).await.unwrap();
    let admin_json: serde_json::Value = serde_json::from_slice(&admin_body).unwrap();
    let admin_token = admin_json["data"]["access_token"].as_str().unwrap().to_string();

    // Register moderator
    let mod_response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/signup")
                .header("content-type", "application/json")
                .body(Body::from(
                    user_registration_with_role(&mod_email, MODERATOR_PASSWORD, "Moderator", "moderator").to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let mod_body = hyper::body::to_bytes(mod_response.into_body()).await.unwrap();
    let mod_json: serde_json::Value = serde_json::from_slice(&mod_body).unwrap();
    let mod_token = mod_json["data"]["access_token"].as_str().unwrap().to_string();

    // Verify all can access their own profiles
    for token in [&user_token, &admin_token, &mod_token] {
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
    }

    // Verify only admin can list users
    let user_list_response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(user_list_response.status(), StatusCode::FORBIDDEN);

    let mod_list_response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", mod_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mod_list_response.status(), StatusCode::FORBIDDEN);

    let admin_list_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/users")
                .header("authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_list_response.status(), StatusCode::OK);
}

/// Test unauthenticated access is properly rejected for all roles
#[tokio::test(flavor = "multi_thread")]
async fn test_e2e_unauthenticated_access_rejected() {
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Try to access protected routes without authentication
    let routes = vec![
        "/users/me",
        "/users",
    ];

    for route in routes {
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(route)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Route {} should require authentication",
            route
        );
    }
}

mod common;

use vibe_api::modules::auth::jwt::{
    generate_access_token, generate_refresh_token, generate_token_pair,
    validate_access_token, validate_refresh_token, TokenType,
};
use vibe_api::modules::users::model::UserRole;
use uuid::Uuid;

// ============================================================================
// JWT TOKEN GENERATION WITH ROLES
// ============================================================================

#[test]
fn test_generate_access_token_with_user_role() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let token = generate_access_token(&user_id, email, UserRole::User, &config)
        .expect("Failed to generate access token");

    assert!(!token.is_empty());

    // Validate token
    let claims = validate_access_token(&token, &config)
        .expect("Failed to validate token");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email, email);
    assert_eq!(claims.role, UserRole::User);
    assert_eq!(claims.token_type, TokenType::Access);
}

#[test]
fn test_generate_access_token_with_admin_role() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "admin@test.com";

    let token = generate_access_token(&user_id, email, UserRole::Admin, &config)
        .expect("Failed to generate access token");

    let claims = validate_access_token(&token, &config)
        .expect("Failed to validate token");

    assert_eq!(claims.role, UserRole::Admin);
}

#[test]
fn test_generate_access_token_with_moderator_role() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "mod@test.com";

    let token = generate_access_token(&user_id, email, UserRole::Moderator, &config)
        .expect("Failed to generate access token");

    let claims = validate_access_token(&token, &config)
        .expect("Failed to validate token");

    assert_eq!(claims.role, UserRole::Moderator);
}

#[test]
fn test_generate_refresh_token_with_role() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let token = generate_refresh_token(&user_id, email, UserRole::Admin, &config)
        .expect("Failed to generate refresh token");

    assert!(!token.is_empty());

    let claims = validate_refresh_token(&token, &config)
        .expect("Failed to validate token");

    assert_eq!(claims.role, UserRole::Admin);
    assert_eq!(claims.token_type, TokenType::Refresh);
}

#[test]
fn test_generate_token_pair_with_role() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let pair = generate_token_pair(&user_id, email, UserRole::Moderator, &config)
        .expect("Failed to generate token pair");

    assert!(!pair.access_token.is_empty());
    assert!(!pair.refresh_token.is_empty());
    assert_eq!(pair.token_type, "Bearer");
    assert!(pair.expires_in > 0);

    // Validate both tokens contain correct role
    let access_claims = validate_access_token(&pair.access_token, &config)
        .expect("Failed to validate access token");
    assert_eq!(access_claims.role, UserRole::Moderator);

    let refresh_claims = validate_refresh_token(&pair.refresh_token, &config)
        .expect("Failed to validate refresh token");
    assert_eq!(refresh_claims.role, UserRole::Moderator);
}

// ============================================================================
// ROLE CLAIM VALIDATION
// ============================================================================

#[test]
fn test_different_roles_in_different_tokens() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();

    // Generate tokens with different roles
    let user_token = generate_access_token(&user_id, "user@test.com", UserRole::User, &config)
        .expect("Failed to generate user token");

    let admin_token = generate_access_token(&user_id, "admin@test.com", UserRole::Admin, &config)
        .expect("Failed to generate admin token");

    let mod_token = generate_access_token(&user_id, "mod@test.com", UserRole::Moderator, &config)
        .expect("Failed to generate moderator token");

    // Validate each token has correct role
    let user_claims = validate_access_token(&user_token, &config).unwrap();
    assert_eq!(user_claims.role, UserRole::User);

    let admin_claims = validate_access_token(&admin_token, &config).unwrap();
    assert_eq!(admin_claims.role, UserRole::Admin);

    let mod_claims = validate_access_token(&mod_token, &config).unwrap();
    assert_eq!(mod_claims.role, UserRole::Moderator);
}

#[test]
fn test_token_expiration_respected() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let token = generate_access_token(&user_id, email, UserRole::User, &config)
        .expect("Failed to generate token");

    let claims = validate_access_token(&token, &config)
        .expect("Failed to validate token");

    // Verify expiration is in the future
    let now = chrono::Utc::now().timestamp();
    assert!(claims.exp > now);

    // Verify issued at is in the past or now
    assert!(claims.iat <= now + 5); // Allow 5 second buffer for test execution
}

#[test]
fn test_token_issuer_validated() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let token = generate_access_token(&user_id, email, UserRole::User, &config)
        .expect("Failed to generate token");

    let claims = validate_access_token(&token, &config)
        .expect("Failed to validate token");

    assert_eq!(claims.iss, config.issuer);
}

#[test]
fn test_invalid_token_rejected() {
    let config = common::app::test_jwt_config();

    let result = validate_access_token("invalid.token.here", &config);
    assert!(result.is_err());
}

#[test]
fn test_token_with_wrong_secret_rejected() {
    let config1 = common::app::test_jwt_config();
    let mut config2 = config1.clone();
    config2.secret = "different_secret_key".to_string();

    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    // Generate token with config1
    let token = generate_access_token(&user_id, email, UserRole::User, &config1)
        .expect("Failed to generate token");

    // Try to validate with config2 (different secret)
    let result = validate_access_token(&token, &config2);
    assert!(result.is_err());
}

// ============================================================================
// TOKEN TYPE VALIDATION
// ============================================================================

#[test]
fn test_refresh_token_rejected_as_access_token() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let refresh_token = generate_refresh_token(&user_id, email, UserRole::User, &config)
        .expect("Failed to generate refresh token");

    // Try to validate refresh token as access token
    let result = validate_access_token(&refresh_token, &config);
    assert!(result.is_err());
}

#[test]
fn test_access_token_rejected_as_refresh_token() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    let access_token = generate_access_token(&user_id, email, UserRole::User, &config)
        .expect("Failed to generate access token");

    // Try to validate access token as refresh token
    let result = validate_refresh_token(&access_token, &config);
    assert!(result.is_err());
}

// ============================================================================
// ROLE SERIALIZATION/DESERIALIZATION
// ============================================================================

#[test]
fn test_user_role_serialization() {
    use serde_json;

    let role = UserRole::User;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"user\"");

    let role = UserRole::Admin;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"admin\"");

    let role = UserRole::Moderator;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"moderator\"");
}

#[test]
fn test_user_role_deserialization() {
    use serde_json;

    let role: UserRole = serde_json::from_str("\"user\"").unwrap();
    assert_eq!(role, UserRole::User);

    let role: UserRole = serde_json::from_str("\"admin\"").unwrap();
    assert_eq!(role, UserRole::Admin);

    let role: UserRole = serde_json::from_str("\"moderator\"").unwrap();
    assert_eq!(role, UserRole::Moderator);
}

#[test]
fn test_user_role_default() {
    let role = UserRole::default();
    assert_eq!(role, UserRole::User);
}

#[test]
fn test_user_role_display() {
    assert_eq!(UserRole::User.to_string(), "user");
    assert_eq!(UserRole::Admin.to_string(), "admin");
    assert_eq!(UserRole::Moderator.to_string(), "moderator");
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_empty_email_in_token() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();

    // Generate token with empty email (should still work)
    let token = generate_access_token(&user_id, "", UserRole::User, &config)
        .expect("Failed to generate token");

    let claims = validate_access_token(&token, &config).unwrap();
    assert_eq!(claims.email, "");
}

#[test]
fn test_multiple_tokens_for_same_user_different_roles() {
    let config = common::app::test_jwt_config();
    let user_id = Uuid::new_v4();
    let email = "user@test.com";

    // Generate multiple tokens with different roles for the same user
    let token1 = generate_access_token(&user_id, email, UserRole::User, &config)
        .expect("Failed to generate token 1");
    let token2 = generate_access_token(&user_id, email, UserRole::Admin, &config)
        .expect("Failed to generate token 2");

    // Both tokens should be valid but with different roles
    let claims1 = validate_access_token(&token1, &config).unwrap();
    let claims2 = validate_access_token(&token2, &config).unwrap();

    assert_eq!(claims1.sub, claims2.sub);
    assert_eq!(claims1.email, claims2.email);
    assert_eq!(claims1.role, UserRole::User);
    assert_eq!(claims2.role, UserRole::Admin);
}

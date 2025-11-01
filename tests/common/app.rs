use axum::Router;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use vibe_api::{
    config::JwtConfig,
    modules::{auth, users},
};

/// Create a test application instance with in-memory database
pub async fn create_test_app(db_pool: Pool<Sqlite>) -> Router {
    // Test JWT configuration
    let jwt_config = JwtConfig {
        secret: "test_secret_key_for_testing_only_not_for_production".to_string(),
        access_token_expiry_hours: 24,
        refresh_token_expiry_days: 30,
        issuer: "vibe-api-test".to_string(),
    };

    // Create routes
    let app = Router::new()
        .merge(auth::routes(db_pool.clone(), jwt_config.clone()))
        .merge(users::routes(db_pool.clone()));

    app
}

/// Create test JWT config
pub fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "test_secret_key_for_testing_only_not_for_production".to_string(),
        access_token_expiry_hours: 24,
        refresh_token_expiry_days: 30,
        issuer: "vibe-api-test".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::create_test_db;

    #[tokio::test]
    async fn test_create_test_app() {
        let db_pool = create_test_db().await;
        let app = create_test_app(db_pool).await;
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_jwt_config() {
        let config = test_jwt_config();
        assert_eq!(config.access_token_expiry_hours, 24);
        assert_eq!(config.refresh_token_expiry_days, 30);
        assert!(!config.secret.is_empty());
    }
}

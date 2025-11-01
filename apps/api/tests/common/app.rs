// Test app builder
// Provides a configured test application instance

use axum::Router;
use sqlx::{Pool, Postgres};
use vibe_api::{
    config::{DatabaseConfig, JwtConfig, ServerConfig},
    modules::{auth, users},
};

use super::fixtures::TEST_JWT_SECRET;

/// Create a test application with all routes
pub async fn create_test_app(db_pool: Pool<Postgres>) -> Router {
    let jwt_config = create_test_jwt_config();

    Router::new()
        // Note: In real implementation, we'd need to adapt routes to work with SQLite
        // For now, this is a placeholder structure
        .merge(auth::routes(db_pool.clone().into(), jwt_config.clone()))
        .merge(users::routes(db_pool.into()))
}

/// Create test JWT configuration
pub fn create_test_jwt_config() -> JwtConfig {
    JwtConfig {
        secret: TEST_JWT_SECRET.to_string(),
        access_token_expiry_hours: 24,
        refresh_token_expiry_days: 30,
        issuer: "vibe-api-test".to_string(),
    }
}

/// Create test database configuration
pub fn create_test_db_config() -> DatabaseConfig {
    DatabaseConfig {
        url: ":memory:".to_string(),
        max_connections: 1,
        min_connections: 1,
        acquire_timeout_secs: 5,
        idle_timeout_secs: 60,
    }
}

/// Create test server configuration
pub fn create_test_server_config() -> ServerConfig {
    ServerConfig {
        port: 0, // Random port for testing
        host: "127.0.0.1".to_string(),
        cors_origins: vec!["*".to_string()],
        environment: vibe_api::config::Environment::Test,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::database::create_test_db;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_test_app() {
        let db_pool = create_test_db().await;
        let app = create_test_app(db_pool).await;
        // Just verify it compiles and creates
        drop(app);
    }

    #[test]
    fn test_create_test_jwt_config() {
        let config = create_test_jwt_config();
        assert_eq!(config.secret, TEST_JWT_SECRET);
        assert_eq!(config.access_token_expiry_hours, 24);
    }

    #[test]
    fn test_create_test_db_config() {
        let config = create_test_db_config();
        assert_eq!(config.url, ":memory:");
        assert_eq!(config.max_connections, 1);
    }

    #[test]
    fn test_create_test_server_config() {
        let config = create_test_server_config();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 0);
    }
}

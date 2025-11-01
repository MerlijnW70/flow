// Comprehensive test application builder
// Provides a fully configured test server instance for integration testing

use axum::Router;
use once_cell::sync::Lazy;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use vibe_api::config::{Config, JwtConfig};

pub static TEST_CONFIG: Lazy<TestConfig> = Lazy::new(|| TestConfig::load());

#[derive(Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_port: u16,
}

impl TestConfig {
    pub fn load() -> Self {
        // Load .env.test if it exists
        let _ = dotenvy::from_filename(".env.test");

        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/vibe_test".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "test_secret_key".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
        }
    }
}

/// Create a test database pool
pub async fn create_test_db_pool() -> PgPool {
    let config = &*TEST_CONFIG;

    PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .expect("Failed to create test database pool")
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}

/// Clean test database
pub async fn clean_test_db(pool: &PgPool) {
    // Truncate all tables
    let _ = sqlx::query("TRUNCATE TABLE users CASCADE")
        .execute(pool)
        .await;
}

/// Create test JWT config
pub fn create_test_jwt_config() -> Arc<JwtConfig> {
    Arc::new(JwtConfig {
        secret: TEST_CONFIG.jwt_secret.clone(),
        access_token_expiry_hours: 1,
        refresh_token_expiry_days: 7,
        issuer: "vibe-api-test".to_string(),
    })
}

/// Create a minimal test app (just health endpoint)
pub fn create_minimal_test_app() -> Router {
    use axum::routing::get;
    use axum::Json;
    use serde_json::json;

    Router::new()
        .route("/api/health", get(|| async {
            Json(json!({"status": "ok"}))
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads() {
        let config = TestConfig::load();
        assert!(!config.database_url.is_empty());
        assert!(!config.jwt_secret.is_empty());
    }

    #[tokio::test]
    async fn test_create_minimal_app() {
        let app = create_minimal_test_app();
        // Verify it compiles
        drop(app);
    }
}

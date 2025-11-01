use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::{info, warn};

use crate::config::DatabaseConfig;
use crate::utils::error::{AppError, AppResult};

pub mod health;

/// Create a PostgreSQL connection pool
pub async fn create_pool(config: &DatabaseConfig) -> AppResult<PgPool> {
    info!("Creating database connection pool...");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .before_acquire(|_conn, _meta| {
            Box::pin(async move {
                Ok(true)
            })
        })
        .after_release(|_conn, _meta| {
            Box::pin(async move {
                Ok(true)
            })
        })
        .connect(&config.url)
        .await
        .map_err(|e| {
            AppError::Database(format!("Failed to create database pool: {}", e))
        })?;

    info!(
        "Database pool created successfully (max: {}, min: {})",
        config.max_connections, config.min_connections
    );

    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| {
            AppError::Database(format!("Failed to run migrations: {}", e))
        })?;

    info!("Database migrations completed successfully");
    Ok(())
}

/// Test database connectivity
pub async fn test_connection(pool: &PgPool) -> AppResult<()> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            AppError::Database(format!("Database connection test failed: {}", e))
        })?;

    Ok(())
}

/// Close database pool gracefully
pub async fn close_pool(pool: PgPool) {
    info!("Closing database connection pool...");
    pool.close().await;
    info!("Database pool closed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_string_validation() {
        // This test ensures invalid connection strings are rejected
        let invalid_config = DatabaseConfig {
            url: "invalid_connection_string".to_string(),
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
        };

        let result = create_pool(&invalid_config).await;
        assert!(result.is_err());
    }
}

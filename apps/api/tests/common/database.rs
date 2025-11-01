// Test database utilities
// Provides in-memory SQLite databases for isolated testing

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test logging (call once)
pub fn init_test_logging() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    });
}

/// Create a test Postgres database pool for testing
pub async fn create_test_db() -> Pool<Postgres> {
    init_test_logging();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/vibe_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test database");

    pool
}

/// Setup test database schema (Postgres version)
async fn setup_test_schema(pool: &Pool<Postgres>) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_login TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
        CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login);
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create test schema");
}

/// Clean up test database (delete all rows)
pub async fn cleanup_test_db(pool: &Pool<Postgres>) {
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await
        .expect("Failed to clean up test database");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_test_db() {
        let pool = create_test_db().await;
        assert!(pool.size() > 0);
    }

    #[tokio::test]
    async fn test_cleanup_test_db() {
        let pool = create_test_db().await;

        // Insert test data
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name) VALUES (?, ?, ?, ?)"
        )
        .bind("test-id")
        .bind("test@example.com")
        .bind("hash")
        .bind("Test User")
        .execute(&pool)
        .await
        .unwrap();

        // Verify data exists
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);

        // Clean up
        cleanup_test_db(&pool).await;

        // Verify cleanup
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }
}

use sqlx::{Pool, Sqlite, SqlitePool};

/// Create an in-memory SQLite database for testing
pub async fn create_test_db() -> Pool<Sqlite> {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create test database");

    // Create users table with role field
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_login TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_db() {
        let pool = create_test_db().await;

        // Verify table exists
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
            .fetch_one(&pool)
            .await;

        assert!(result.is_ok());
    }
}

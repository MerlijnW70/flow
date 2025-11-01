// Database Integration Tests
// Validates database connectivity, migrations, and query performance

mod common;

use common::{create_test_db_pool, run_migrations, clean_test_db};
use sqlx::Row;
use std::time::Instant;

#[tokio::test]
async fn test_database_connection() {
    // Arrange & Act
    let pool = create_test_db_pool().await;

    // Assert
    assert!(pool.size() > 0, "Pool should have at least one connection");
}

#[tokio::test]
async fn test_simple_query() {
    // Arrange
    let pool = create_test_db_pool().await;

    // Act
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute query");

    // Assert
    assert_eq!(result.0, 1);
}

#[tokio::test]
async fn test_query_latency() {
    // Arrange
    let pool = create_test_db_pool().await;

    // Act
    let start = Instant::now();
    let _: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute query");
    let duration = start.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 50,
        "Query took {}ms, expected < 50ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_migrations_run_successfully() {
    // Arrange
    let pool = create_test_db_pool().await;

    // Act
    let result = sqlx::migrate!("./migrations")
        .run(&pool)
        .await;

    // Assert
    assert!(result.is_ok(), "Migrations should run successfully");
}

#[tokio::test]
async fn test_migrations_are_idempotent() {
    // Arrange
    let pool = create_test_db_pool().await;

    // Act - Run migrations twice
    let result1 = sqlx::migrate!("./migrations").run(&pool).await;
    let result2 = sqlx::migrate!("./migrations").run(&pool).await;

    // Assert
    assert!(result1.is_ok(), "First migration run should succeed");
    assert!(result2.is_ok(), "Second migration run should succeed (idempotent)");
}

#[tokio::test]
async fn test_users_table_exists() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;

    // Act
    let result = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_name = 'users'"
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query information_schema");

    // Assert
    assert!(result.is_some(), "users table should exist after migrations");
}

#[tokio::test]
async fn test_clean_test_database() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;

    // Insert test data
    let _ = sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role) VALUES (gen_random_uuid(), 'test@example.com', 'hash', 'Test User', 'user')"
    )
    .execute(&pool)
    .await;

    // Act
    clean_test_db(&pool).await;

    // Assert
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count.0, 0, "Users table should be empty after cleanup");
}

#[tokio::test]
async fn test_connection_pool_limits() {
    // Arrange
    let pool = create_test_db_pool().await;

    // Act - Try to get multiple connections
    let conn1 = pool.acquire().await;
    let conn2 = pool.acquire().await;

    // Assert
    assert!(conn1.is_ok(), "Should be able to acquire first connection");
    assert!(conn2.is_ok(), "Should be able to acquire second connection");
}

#[tokio::test]
async fn test_concurrent_queries() {
    // Arrange
    let pool = create_test_db_pool().await;

    // Act - Run 10 queries concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            sqlx::query_as::<_, (i32,)>("SELECT 1")
                .fetch_one(&pool_clone)
                .await
        });
        handles.push(handle);
    }

    // Wait for all queries to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert
    for result in results {
        assert!(result.is_ok(), "Concurrent query failed");
        let query_result = result.unwrap();
        assert!(query_result.is_ok(), "Query execution failed");
    }
}

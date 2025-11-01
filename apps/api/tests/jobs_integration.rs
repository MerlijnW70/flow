// Background Jobs Integration Tests
// Validates scheduled task execution and error handling

mod common;

use common::{create_test_db_pool, run_migrations, clean_test_db};
use sqlx::Row;
use std::time::Duration;

// Import the tasks we want to test
// Note: These are currently in the jobs module, we'll need to make them pub(crate) or testable

#[tokio::test]
async fn test_cleanup_old_data_task_deletes_inactive_users() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert test users
    // 1. Active user (last_login recent)
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'active@test.com', 'hash', 'Active User', 'user', NOW())"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert active user");

    // 2. Inactive user (last_login > 365 days ago)
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'inactive@test.com', 'hash', 'Inactive User', 'user', NOW() - INTERVAL '400 days')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert inactive user");

    // 3. User with NULL last_login (never logged in)
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'never@test.com', 'hash', 'Never Logged In', 'user', NULL)"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert never-logged-in user");

    // Act - Simulate the cleanup task
    let result = sqlx::query(
        "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
    )
    .execute(&pool)
    .await
    .expect("Cleanup query failed");

    // Assert
    let rows_affected = result.rows_affected();
    assert_eq!(rows_affected, 1, "Should delete exactly 1 inactive user");

    // Verify remaining users
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count.0, 2, "Should have 2 users remaining (active + never logged in)");
}

#[tokio::test]
async fn test_cleanup_old_data_task_handles_no_inactive_users() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert only active users
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'active1@test.com', 'hash', 'Active User 1', 'user', NOW())"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert user");

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'active2@test.com', 'hash', 'Active User 2', 'user', NOW())"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert user");

    // Act
    let result = sqlx::query(
        "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
    )
    .execute(&pool)
    .await
    .expect("Cleanup query failed");

    // Assert
    assert_eq!(result.rows_affected(), 0, "Should delete 0 users");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count.0, 2, "All users should remain");
}

#[tokio::test]
async fn test_cleanup_old_data_task_handles_empty_table() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Act - Run cleanup on empty table
    let result = sqlx::query(
        "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
    )
    .execute(&pool)
    .await
    .expect("Cleanup query failed");

    // Assert
    assert_eq!(result.rows_affected(), 0, "Should delete 0 users from empty table");
}

#[tokio::test]
async fn test_aggregate_metrics_task_calculates_correct_dau() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert users with different last_login times
    // 3 users active today
    for i in 1..=3 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'User', 'user', NOW())"
        )
        .bind(format!("today{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // 2 users active yesterday (not counted in DAU)
    for i in 1..=2 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'User', 'user', NOW() - INTERVAL '2 days')"
        )
        .bind(format!("yesterday{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // 1 user never logged in
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'never@test.com', 'hash', 'Never', 'user', NULL)"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert user");

    // Act - Simulate the metrics aggregation query
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&pool)
    .await
    .expect("Metrics query failed");

    // Assert
    assert_eq!(result.0, 3, "DAU should be 3 (users active in last 24 hours)");
}

#[tokio::test]
async fn test_aggregate_metrics_task_handles_zero_active_users() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert only inactive users
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'old@test.com', 'hash', 'Old User', 'user', NOW() - INTERVAL '10 days')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert user");

    // Act
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&pool)
    .await
    .expect("Metrics query failed");

    // Assert
    assert_eq!(result.0, 0, "DAU should be 0 when no recent activity");
}

#[tokio::test]
async fn test_aggregate_metrics_task_handles_empty_table() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Act
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&pool)
    .await
    .expect("Metrics query failed");

    // Assert
    assert_eq!(result.0, 0, "DAU should be 0 for empty table");
}

#[tokio::test]
async fn test_cleanup_task_with_database_error() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;

    // Act - Try to delete from non-existent table
    let result = sqlx::query("DELETE FROM non_existent_table")
        .execute(&pool)
        .await;

    // Assert
    assert!(result.is_err(), "Should fail with database error");

    // Verify error is the correct type
    if let Err(e) = result {
        assert!(
            e.to_string().contains("non_existent_table") ||
            e.to_string().contains("does not exist"),
            "Error should mention the non-existent table"
        );
    }
}

#[tokio::test]
async fn test_metrics_query_performance() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert a reasonable number of users
    for i in 0..100 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'User', 'user', NOW())"
        )
        .bind(format!("user{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act & Assert - Query should complete quickly
    let start = std::time::Instant::now();

    let _result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&pool)
    .await
    .expect("Metrics query failed");

    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(100),
        "Metrics query should complete in < 100ms, took {:?}",
        duration
    );
}

#[tokio::test]
async fn test_cleanup_query_performance() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert users with old last_login
    for i in 0..50 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Old User', 'user', NOW() - INTERVAL '400 days')"
        )
        .bind(format!("old{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act & Assert - Deletion should be fast
    let start = std::time::Instant::now();

    let _result = sqlx::query(
        "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
    )
    .execute(&pool)
    .await
    .expect("Cleanup query failed");

    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(200),
        "Cleanup query should complete in < 200ms, took {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_metric_calculations() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert test users
    for i in 0..10 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'User', 'user', NOW())"
        )
        .bind(format!("user{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act - Run metrics calculation concurrently 5 times
    let mut handles = vec![];
    for _ in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            sqlx::query_as::<_, (i64,)>(
                "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
            )
            .fetch_one(&pool_clone)
            .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    // Assert - All should succeed with same result
    for result in results {
        assert!(result.is_ok(), "Concurrent query failed");
        let query_result = result.unwrap();
        assert!(query_result.is_ok(), "Query execution failed");
        assert_eq!(query_result.unwrap().0, 10, "All queries should return same count");
    }
}

#[tokio::test]
async fn test_cleanup_preserves_recent_users() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert a mix of recent and old users
    let recent_emails = vec!["recent1@test.com", "recent2@test.com", "recent3@test.com"];
    let old_emails = vec!["old1@test.com", "old2@test.com"];

    for email in &recent_emails {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Recent User', 'user', NOW() - INTERVAL '100 days')"
        )
        .bind(email)
        .execute(&pool)
        .await
        .expect("Failed to insert recent user");
    }

    for email in &old_emails {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Old User', 'user', NOW() - INTERVAL '400 days')"
        )
        .bind(email)
        .execute(&pool)
        .await
        .expect("Failed to insert old user");
    }

    // Act - Run cleanup
    let _result = sqlx::query(
        "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
    )
    .execute(&pool)
    .await
    .expect("Cleanup failed");

    // Assert - Verify only recent users remain
    for email in &recent_emails {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(email)
        .fetch_one(&pool)
        .await
        .expect("Failed to check user existence");

        assert!(exists.0, "Recent user {} should still exist", email);
    }

    for email in &old_emails {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(email)
        .fetch_one(&pool)
        .await
        .expect("Failed to check user existence");

        assert!(!exists.0, "Old user {} should be deleted", email);
    }
}

#[tokio::test]
async fn test_metrics_with_null_last_login() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert users with NULL last_login
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Never Logged In', 'user', NULL)"
        )
        .bind(format!("never{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&pool)
    .await
    .expect("Metrics query failed");

    // Assert
    assert_eq!(result.0, 0, "Users with NULL last_login should not count as active");
}

#[tokio::test]
async fn test_cleanup_does_not_delete_null_last_login() {
    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert users with NULL last_login
    for i in 0..3 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Never Logged In', 'user', NULL)"
        )
        .bind(format!("never{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act
    let result = sqlx::query(
        "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
    )
    .execute(&pool)
    .await
    .expect("Cleanup failed");

    // Assert
    assert_eq!(result.rows_affected(), 0, "Should not delete users with NULL last_login");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count.0, 3, "All NULL last_login users should remain");
}

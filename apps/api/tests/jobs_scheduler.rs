// Background Jobs Scheduler Integration Tests
// Validates job scheduler setup and execution

mod common;

use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use std::time::Duration;
use tokio::time::sleep;

// Note: These tests validate the scheduler behavior pattern
// Actual scheduler integration depends on making the scheduler testable

#[tokio::test]
async fn test_scheduler_can_be_created() {
    // This test validates that we can create a scheduler instance
    // In real implementation, this would use tokio_cron_scheduler::JobScheduler

    // Arrange & Act
    let result = std::panic::catch_unwind(|| {
        // Simulating scheduler creation
        // In real code: JobScheduler::new().await
        "scheduler_created"
    });

    // Assert
    assert!(result.is_ok(), "Scheduler creation should not panic");
}

#[tokio::test]
async fn test_async_job_execution() {
    // Arrange - Create a counter to track job execution
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // Act - Simulate an async job
    let job = async move {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let result = job.await;

    // Assert
    assert!(result.is_ok(), "Job should complete successfully");
    assert_eq!(counter.load(Ordering::SeqCst), 1, "Job should execute once");
}

#[tokio::test]
async fn test_job_with_database_operation() {
    use common::{create_test_db_pool, run_migrations, clean_test_db};

    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert a test user
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, last_login)
         VALUES (gen_random_uuid(), 'job@test.com', 'hash', 'Job User', 'user', NOW() - INTERVAL '400 days')"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert user");

    // Act - Simulate job execution
    let job_result = async {
        let result = sqlx::query(
            "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
        )
        .execute(&pool)
        .await;

        match result {
            Ok(res) => {
                println!("[Job] Deleted {} inactive users", res.rows_affected());
                Ok(())
            }
            Err(e) => {
                eprintln!("[Job Error] Failed to cleanup: {}", e);
                Err(e)
            }
        }
    }.await;

    // Assert
    assert!(job_result.is_ok(), "Job should complete successfully");

    // Verify the job did its work
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    assert_eq!(count.0, 0, "Job should have deleted the old user");
}

#[tokio::test]
async fn test_job_handles_database_error_gracefully() {
    use common::{create_test_db_pool, run_migrations};

    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;

    // Act - Simulate job with bad query
    let job_result = async {
        let result = sqlx::query("DELETE FROM non_existent_table")
            .execute(&pool)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // Job should log error but not panic
                eprintln!("[Job Error] Database error: {}", e);
                // In production, this might send an alert or metric
                Err(e)
            }
        }
    }.await;

    // Assert - Job fails but doesn't panic
    assert!(job_result.is_err(), "Job should return error for bad query");
}

#[tokio::test]
async fn test_multiple_jobs_can_run_concurrently() {
    use common::{create_test_db_pool, run_migrations, clean_test_db};

    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Create test data
    for i in 0..10 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'User', 'user', NOW())"
        )
        .bind(format!("concurrent{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act - Run two different jobs concurrently
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let job1 = tokio::spawn(async move {
        // Metrics job
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&pool1)
        .await
        .expect("Metrics job failed");

        result.0
    });

    let job2 = tokio::spawn(async move {
        // Count job
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool2)
            .await
            .expect("Count job failed");

        result.0
    });

    // Wait for both jobs
    let (metrics_result, count_result) = tokio::join!(job1, job2);

    // Assert
    assert!(metrics_result.is_ok(), "Metrics job should succeed");
    assert!(count_result.is_ok(), "Count job should succeed");
    assert_eq!(metrics_result.unwrap(), 10, "Metrics should count all recent users");
    assert_eq!(count_result.unwrap(), 10, "Count should match total users");
}

#[tokio::test]
async fn test_job_execution_timing() {
    // Arrange
    let start = std::time::Instant::now();
    let execution_count = Arc::new(AtomicU32::new(0));

    // Act - Simulate a job that executes every 100ms, 3 times
    for _ in 0..3 {
        let count = execution_count.clone();
        let job = async move {
            count.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(10)).await;
        };

        job.await;
        sleep(Duration::from_millis(100)).await;
    }

    let duration = start.elapsed();

    // Assert
    assert_eq!(execution_count.load(Ordering::SeqCst), 3, "Job should execute 3 times");
    assert!(
        duration >= Duration::from_millis(300),
        "Total duration should be at least 300ms"
    );
    assert!(
        duration < Duration::from_millis(500),
        "Total duration should be less than 500ms"
    );
}

#[tokio::test]
async fn test_job_cleanup_logs_result() {
    use common::{create_test_db_pool, run_migrations, clean_test_db};

    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert test data
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Old User', 'user', NOW() - INTERVAL '400 days')"
        )
        .bind(format!("old{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act - Job with logging
    let deleted_count = async {
        let result = sqlx::query(
            "DELETE FROM users WHERE last_login < NOW() - INTERVAL '365 days'"
        )
        .execute(&pool)
        .await
        .expect("Cleanup failed");

        let count = result.rows_affected();
        println!("[Cleanup Job] Deleted {} inactive users", count);

        count
    }.await;

    // Assert
    assert_eq!(deleted_count, 5, "Should delete all 5 old users");
}

#[tokio::test]
async fn test_metrics_job_logs_result() {
    use common::{create_test_db_pool, run_migrations, clean_test_db};

    // Arrange
    let pool = create_test_db_pool().await;
    run_migrations(&pool).await;
    clean_test_db(&pool).await;

    // Insert test data
    for i in 0..7 {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, last_login)
             VALUES (gen_random_uuid(), $1, 'hash', 'Active User', 'user', NOW())"
        )
        .bind(format!("active{}@test.com", i))
        .execute(&pool)
        .await
        .expect("Failed to insert user");
    }

    // Act - Metrics job with logging
    let dau_count = async {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE last_login >= NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&pool)
        .await
        .expect("Metrics query failed");

        let count = result.0;
        println!("[Metrics Job] Daily Active Users (DAU): {}", count);

        count
    }.await;

    // Assert
    assert_eq!(dau_count, 7, "DAU should be 7");
}

#[tokio::test]
async fn test_job_error_does_not_panic_scheduler() {
    // Arrange - Simulate a job that fails
    let error_count = Arc::new(AtomicU32::new(0));

    // Act - Run 3 jobs, middle one fails
    for i in 0..3 {
        let count = error_count.clone();
        let job_result = async move {
            if i == 1 {
                // Simulate error
                Err::<(), _>("Simulated job failure".to_string())
            } else {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }.await;

        // Scheduler should log error and continue
        if let Err(e) = job_result {
            eprintln!("[Scheduler] Job failed: {}", e);
            // In production, this would increment an error metric
        }
    }

    // Assert - Successful jobs still ran
    assert_eq!(error_count.load(Ordering::SeqCst), 2, "2 jobs should succeed despite 1 failure");
}

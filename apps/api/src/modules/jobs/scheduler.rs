use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::utils::error::{AppError, AppResult};

use super::tasks;

pub async fn start(db_pool: PgPool) -> AppResult<()> {
    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| AppError::InternalServer(format!("Failed to create scheduler: {}", e)))?;

    // Example: Run cleanup task every day at midnight
    let cleanup_job = Job::new_async("0 0 0 * * *", move |_uuid, _lock| {
        let pool = db_pool.clone();
        Box::pin(async move {
            info!("Running daily cleanup task");
            match tasks::cleanup_old_data(pool).await {
                Ok(_) => info!("Cleanup task completed successfully"),
                Err(e) => error!("Cleanup task failed: {}", e),
            }
        })
    })
    .map_err(|e| AppError::InternalServer(format!("Failed to create cleanup job: {}", e)))?;

    scheduler
        .add(cleanup_job)
        .await
        .map_err(|e| AppError::InternalServer(format!("Failed to add cleanup job: {}", e)))?;

    // Example: Run metrics aggregation every hour
    let db_pool_clone = db_pool.clone();
    let metrics_job = Job::new_async("0 0 * * * *", move |_uuid, _lock| {
        let pool = db_pool_clone.clone();
        Box::pin(async move {
            info!("Running hourly metrics aggregation");
            match tasks::aggregate_metrics(pool).await {
                Ok(_) => info!("Metrics aggregation completed successfully"),
                Err(e) => error!("Metrics aggregation failed: {}", e),
            }
        })
    })
    .map_err(|e| AppError::InternalServer(format!("Failed to create metrics job: {}", e)))?;

    scheduler
        .add(metrics_job)
        .await
        .map_err(|e| AppError::InternalServer(format!("Failed to add metrics job: {}", e)))?;

    // Start the scheduler
    scheduler
        .start()
        .await
        .map_err(|e| AppError::InternalServer(format!("Failed to start scheduler: {}", e)))?;

    info!("Job scheduler started successfully");

    // Keep scheduler alive by spawning it in a background task
    tokio::spawn(async move {
        // Scheduler will run in the background
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    Ok(())
}

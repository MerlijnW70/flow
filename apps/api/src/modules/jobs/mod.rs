pub mod scheduler;
pub mod tasks;

use sqlx::PgPool;
use crate::utils::error::AppResult;

pub async fn start_scheduler(db_pool: PgPool) -> AppResult<()> {
    scheduler::start(db_pool).await
}

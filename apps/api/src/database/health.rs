use sqlx::PgPool;
use serde::{Deserialize, Serialize};

use crate::utils::error::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub connections: ConnectionStats,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total: u32,
    pub idle: u32,
}

pub async fn check_health(pool: &PgPool) -> AppResult<DatabaseHealth> {
    let start = std::time::Instant::now();

    // Test database connectivity with a simple query
    let result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await;

    let latency_ms = start.elapsed().as_millis() as u64;

    let status = match result {
        Ok(_) => {
            if latency_ms > 1000 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            }
        }
        Err(_) => HealthStatus::Unhealthy,
    };

    let connections = ConnectionStats {
        total: pool.size(),
        idle: pool.num_idle() as u32,
    };

    Ok(DatabaseHealth {
        status,
        connections,
        latency_ms: Some(latency_ms),
    })
}

// Public library interface for vibe-api
// This exposes modules that can be used by tests and other binaries

pub mod config;
pub mod database;
pub mod metrics;
pub mod middleware;
pub mod modules;
pub mod utils;

// Re-export commonly used types
pub use config::Config;
pub use utils::error::{AppError, AppResult};

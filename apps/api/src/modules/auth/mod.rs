pub mod jwt;
pub mod hash;
pub mod service;
pub mod model;
pub mod routes;
pub mod middleware;
pub mod role_guard;

pub use routes::routes;
pub use middleware::AuthMiddleware;
pub use role_guard::{require_admin, require_moderator, require_role};

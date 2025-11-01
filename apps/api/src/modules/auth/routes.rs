use axum::{
    routing::post,
    Router, Json, extract::State,
};
use sqlx::PgPool;
use std::sync::Arc;
use validator::Validate;

use crate::config::JwtConfig;
use crate::utils::{
    error::{AppError, AppResult},
    response::{created, ApiResponse},
    validation::validate_struct,
};

use super::model::{AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest};
use super::service::AuthService;

#[derive(Clone)]
struct AuthState {
    service: Arc<AuthService>,
}

pub fn routes(db_pool: PgPool, jwt_config: JwtConfig) -> Router {
    let service = Arc::new(AuthService::new(db_pool, jwt_config));
    let state = AuthState { service };

    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .with_state(state)
}

async fn register(
    State(state): State<AuthState>,
    Json(request): Json<RegisterRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // Validate request
    validate_struct(&request)?;

    // Register user
    let response = state.service.register(request).await?;

    Ok(created(response))
}

async fn login(
    State(state): State<AuthState>,
    Json(request): Json<LoginRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // Validate request
    validate_struct(&request)?;

    // Login user
    let response = state.service.login(request).await?;

    Ok(ApiResponse::success(response))
}

async fn refresh_token(
    State(state): State<AuthState>,
    Json(request): Json<RefreshTokenRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // Refresh token
    let response = state.service.refresh_token(request).await?;

    Ok(ApiResponse::success(response))
}

use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{delete, get, patch, put},
    Extension, Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::config::JwtConfig;
use crate::modules::auth::{
    jwt::Claims,
    middleware::auth_middleware,
    role_guard::require_admin,
};
use crate::utils::{
    error::{AppError, AppResult},
    response::{no_content, ApiResponse, PaginatedResponse},
    validation::validate_struct,
};

use super::model::{ChangePasswordRequest, UpdateUserRequest, UserResponse};
use super::service::UserService;

#[derive(Clone)]
struct UserState {
    service: Arc<UserService>,
    jwt_config: Arc<JwtConfig>,
}

#[derive(Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_per_page")]
    per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

pub fn routes(db_pool: PgPool) -> Router {
    // Note: JWT config will be provided at runtime via layer
    // For now, we create a placeholder that will be replaced in main.rs
    let jwt_config = Arc::new(JwtConfig {
        secret: String::new(),
        access_token_expiry_hours: 24,
        refresh_token_expiry_days: 30,
        issuer: String::new(),
    });

    let service = Arc::new(UserService::new(db_pool));
    let state = UserState { service, jwt_config: jwt_config.clone() };

    // Public/authenticated routes (any authenticated user)
    let authenticated_routes = Router::new()
        .route("/users/me", get(get_current_user))
        .route("/users/me", patch(update_current_user))
        .route("/users/me", delete(delete_current_user))
        .route("/users/me/password", put(change_password))
        .layer(middleware::from_fn_with_state(jwt_config.clone(), auth_middleware));

    // Admin-only routes
    let admin_routes = Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user_by_id))
        .route("/users/:id", delete(delete_user_by_id))
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn_with_state(jwt_config, auth_middleware));

    Router::new()
        .merge(authenticated_routes)
        .merge(admin_routes)
        .with_state(state)
}

async fn get_current_user(
    State(state): State<UserState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<impl axum::response::IntoResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    let user = state.service.get_by_id(&user_id).await?;

    Ok(ApiResponse::success(user))
}

async fn get_user_by_id(
    State(state): State<UserState>,
    Path(user_id): Path<Uuid>,
) -> AppResult<impl axum::response::IntoResponse> {
    let user = state.service.get_by_id(&user_id).await?;
    Ok(ApiResponse::success(user))
}

async fn update_current_user(
    State(state): State<UserState>,
    Extension(claims): Extension<Claims>,
    Json(update_request): Json<UpdateUserRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    validate_struct(&update_request)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    let user = state.service.update(&user_id, update_request).await?;

    Ok(ApiResponse::success(user))
}

async fn delete_current_user(
    State(state): State<UserState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<impl axum::response::IntoResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    state.service.delete(&user_id).await?;

    Ok(no_content())
}

async fn change_password(
    State(state): State<UserState>,
    Extension(claims): Extension<Claims>,
    Json(password_request): Json<ChangePasswordRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    validate_struct(&password_request)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID".to_string()))?;

    state.service.change_password(&user_id, password_request).await?;

    Ok(ApiResponse::with_message(
        (),
        "Password changed successfully".to_string(),
    ))
}

async fn list_users(
    State(state): State<UserState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let (users, total) = state.service.list(pagination.page, pagination.per_page).await?;

    Ok(PaginatedResponse::new(
        users,
        pagination.page,
        pagination.per_page,
        total,
    ))
}

async fn delete_user_by_id(
    State(state): State<UserState>,
    Path(user_id): Path<Uuid>,
) -> AppResult<impl axum::response::IntoResponse> {
    state.service.delete(&user_id).await?;
    Ok(no_content())
}

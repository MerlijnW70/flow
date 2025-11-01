use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::modules::users::model::UserRole;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "user@example.com")]
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[schema(example = "SecurePass123!")]
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[schema(example = "John Doe")]
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[schema(example = "user")]
    // Optional role (defaults to User if not provided)
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[schema(example = "SecurePass123!")]
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

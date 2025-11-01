use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalServer(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("File too large")]
    FileTooLarge,

    #[error("Unsupported media type")]
    UnsupportedMediaType,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                self.to_string(),
            ),
            AppError::Authentication(_) => (
                StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_ERROR",
                self.to_string(),
            ),
            AppError::Authorization(_) => (
                StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                self.to_string(),
            ),
            AppError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                self.to_string(),
            ),
            AppError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                self.to_string(),
            ),
            AppError::Conflict(_) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                self.to_string(),
            ),
            AppError::BadRequest(_) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                self.to_string(),
            ),
            AppError::InternalServer(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "An internal server error occurred".to_string(),
            ),
            AppError::ExternalService(_) => (
                StatusCode::BAD_GATEWAY,
                "EXTERNAL_SERVICE_ERROR",
                self.to_string(),
            ),
            AppError::Configuration(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIGURATION_ERROR",
                "Configuration error occurred".to_string(),
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Rate limit exceeded. Please try again later.".to_string(),
            ),
            AppError::FileTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "FILE_TOO_LARGE",
                "File size exceeds maximum allowed size".to_string(),
            ),
            AppError::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "Unsupported media type".to_string(),
            ),
        };

        // Log internal errors
        if matches!(
            self,
            AppError::Database(_) | AppError::InternalServer(_) | AppError::Configuration(_)
        ) {
            tracing::error!("Internal error: {:?}", self);
        }

        let body = Json(ErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                details: None,
            },
        });

        (status, body).into_response()
    }
}

// Conversion from sqlx errors
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                AppError::NotFound("Resource not found".to_string())
            }
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violations
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        // PostgreSQL unique violation
                        return AppError::Conflict(
                            "Resource already exists".to_string()
                        );
                    }
                }
                AppError::Database(db_err.to_string())
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}

// Conversion from validation errors
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err.to_string())
    }
}

// Conversion from config errors
impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::Configuration(err.to_string())
    }
}

// Conversion from generic errors
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServer(err.to_string())
    }
}

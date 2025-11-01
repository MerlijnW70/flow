use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::config::JwtConfig;
use crate::modules::auth::jwt::{validate_access_token, Claims};
use crate::utils::error::AppError;

#[derive(Clone)]
pub struct AuthMiddleware {
    pub jwt_config: Arc<JwtConfig>,
}

impl AuthMiddleware {
    pub fn new(jwt_config: JwtConfig) -> Self {
        Self {
            jwt_config: Arc::new(jwt_config),
        }
    }
}

/// Extract JWT token from Authorization header
fn extract_token(headers: &HeaderMap) -> Result<String, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Authentication("Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Authentication(
            "Invalid authorization header format".to_string(),
        ));
    }

    Ok(auth_header[7..].to_string())
}

/// Middleware function to validate JWT token
pub async fn auth_middleware(
    State(jwt_config): State<Arc<JwtConfig>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(request.headers())?;
    let claims = validate_access_token(&token, &jwt_config)?;

    // Insert claims into request extensions so handlers can access them
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extension trait to easily extract claims from requests
pub trait ClaimsExtractor {
    fn claims(&self) -> Result<&Claims, AppError>;
}

impl ClaimsExtractor for Request {
    fn claims(&self) -> Result<&Claims, AppError> {
        self.extensions()
            .get::<Claims>()
            .ok_or_else(|| AppError::Authentication("No claims found in request".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_token_success() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer test_token_123"),
        );

        let token = extract_token(&headers).expect("Failed to extract token");
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_extract_token_missing_header() {
        let headers = HeaderMap::new();
        let result = extract_token(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("InvalidFormat token"));

        let result = extract_token(&headers);
        assert!(result.is_err());
    }
}

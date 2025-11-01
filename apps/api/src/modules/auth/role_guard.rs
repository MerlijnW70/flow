use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::modules::auth::jwt::Claims;
use crate::modules::users::model::UserRole;
use crate::utils::error::AppError;

/// Role guard middleware - checks if user has required role
pub async fn require_role(
    required_roles: Vec<UserRole>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract claims from request extensions (set by auth_middleware)
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authentication("No authentication found".to_string()))?;

    // Check if user has one of the required roles
    if !required_roles.contains(&claims.role) {
        return Err(AppError::Authorization(
            format!("Insufficient permissions. Required: {:?}, Have: {:?}", required_roles, claims.role)
        ));
    }

    Ok(next.run(request).await)
}

/// Require admin role
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    require_role(vec![UserRole::Admin], request, next).await
}

/// Require admin or moderator role
pub async fn require_moderator(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    require_role(vec![UserRole::Admin, UserRole::Moderator], request, next).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request as HttpRequest, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn test_handler() -> impl IntoResponse {
        (StatusCode::OK, "success")
    }

    fn create_test_claims(role: UserRole) -> Claims {
        use crate::modules::auth::jwt::TokenType;
        use chrono::Utc;

        Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            role,
            exp: (Utc::now().timestamp() + 3600),
            iat: Utc::now().timestamp(),
            iss: "test".to_string(),
            token_type: TokenType::Access,
        }
    }

    #[tokio::test]
    async fn test_require_admin_with_admin_role() {
        let app = Router::new()
            .route("/admin", get(test_handler))
            .layer(middleware::from_fn(require_admin));

        let mut request = HttpRequest::builder()
            .uri("/admin")
            .body(Body::empty())
            .unwrap();

        // Add admin claims to request extensions
        request.extensions_mut().insert(create_test_claims(UserRole::Admin));

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_require_admin_with_user_role() {
        let app = Router::new()
            .route("/admin", get(test_handler))
            .layer(middleware::from_fn(require_admin));

        let mut request = HttpRequest::builder()
            .uri("/admin")
            .body(Body::empty())
            .unwrap();

        // Add user claims to request extensions
        request.extensions_mut().insert(create_test_claims(UserRole::User));

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_require_moderator_with_moderator_role() {
        let app = Router::new()
            .route("/moderate", get(test_handler))
            .layer(middleware::from_fn(require_moderator));

        let mut request = HttpRequest::builder()
            .uri("/moderate")
            .body(Body::empty())
            .unwrap();

        // Add moderator claims to request extensions
        request.extensions_mut().insert(create_test_claims(UserRole::Moderator));

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_require_moderator_with_admin_role() {
        let app = Router::new()
            .route("/moderate", get(test_handler))
            .layer(middleware::from_fn(require_moderator));

        let mut request = HttpRequest::builder()
            .uri("/moderate")
            .body(Body::empty())
            .unwrap();

        // Admin should also have moderator permissions
        request.extensions_mut().insert(create_test_claims(UserRole::Admin));

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_role_guard_without_claims() {
        let app = Router::new()
            .route("/admin", get(test_handler))
            .layer(middleware::from_fn(require_admin));

        let request = HttpRequest::builder()
            .uri("/admin")
            .body(Body::empty())
            .unwrap();

        // No claims in request extensions
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

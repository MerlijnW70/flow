use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::modules::users::model::UserRole;
use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,
    pub role: UserRole,     // User role
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub iss: String,        // Issuer
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Generate a JWT access token with role
pub fn generate_access_token(
    user_id: &Uuid,
    email: &str,
    role: UserRole,
    config: &JwtConfig,
) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.access_token_expiry_hours))
        .ok_or_else(|| AppError::InternalServer("Invalid expiration time".to_string()))?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role,  // Include role
        exp: expiration,
        iat: Utc::now().timestamp(),
        iss: config.issuer.clone(),
        token_type: TokenType::Access,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Authentication(format!("Failed to generate token: {}", e)))
}

/// Generate a JWT refresh token with role
pub fn generate_refresh_token(
    user_id: &Uuid,
    email: &str,
    role: UserRole,
    config: &JwtConfig,
) -> AppResult<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(config.refresh_token_expiry_days))
        .ok_or_else(|| AppError::InternalServer("Invalid expiration time".to_string()))?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role,  // Include role
        exp: expiration,
        iat: Utc::now().timestamp(),
        iss: config.issuer.clone(),
        token_type: TokenType::Refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Authentication(format!("Failed to generate refresh token: {}", e)))
}

/// Generate both access and refresh tokens with role
pub fn generate_token_pair(
    user_id: &Uuid,
    email: &str,
    role: UserRole,
    config: &JwtConfig,
) -> AppResult<TokenPair> {
    let access_token = generate_access_token(user_id, email, role, config)?;
    let refresh_token = generate_refresh_token(user_id, email, role, config)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.access_token_expiry_hours * 3600,
    })
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str, config: &JwtConfig) -> AppResult<Claims> {
    let mut validation = Validation::default();
    validation.set_issuer(&[config.issuer.clone()]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        AppError::Authentication(format!("Invalid token: {}", e))
    })
}

/// Validate that the token is an access token
pub fn validate_access_token(token: &str, config: &JwtConfig) -> AppResult<Claims> {
    let claims = validate_token(token, config)?;

    if claims.token_type != TokenType::Access {
        return Err(AppError::Authentication(
            "Invalid token type".to_string(),
        ));
    }

    Ok(claims)
}

/// Validate that the token is a refresh token
pub fn validate_refresh_token(token: &str, config: &JwtConfig) -> AppResult<Claims> {
    let claims = validate_token(token, config)?;

    if claims.token_type != TokenType::Refresh {
        return Err(AppError::Authentication(
            "Invalid token type".to_string(),
        ));
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_for_testing_purposes".to_string(),
            access_token_expiry_hours: 24,
            refresh_token_expiry_days: 30,
            issuer: "vibe-api-test".to_string(),
        }
    }

    #[test]
    fn test_generate_and_validate_access_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let role = UserRole::User;

        let token = generate_access_token(&user_id, email, role, &config)
            .expect("Failed to generate token");

        let claims = validate_access_token(&token, &config)
            .expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, UserRole::User);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_generate_token_pair() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let role = UserRole::Admin;

        let pair = generate_token_pair(&user_id, email, role, &config)
            .expect("Failed to generate token pair");

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
    }
}

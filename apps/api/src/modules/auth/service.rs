use sqlx::PgPool;
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::modules::users::model::User;
use crate::utils::error::{AppError, AppResult};

use super::hash::{hash_password, verify_password};
use super::jwt::{generate_token_pair, validate_refresh_token};
use super::model::{AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest, UserInfo};

pub struct AuthService {
    db_pool: PgPool,
    jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(db_pool: PgPool, jwt_config: JwtConfig) -> Self {
        Self { db_pool, jwt_config }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> AppResult<AuthResponse> {
        // Check if user already exists
        let existing_user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(&request.email)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::Conflict("User with this email already exists".to_string()));
        }

        // Hash password
        let password_hash = hash_password(&request.password)?;

        // Create user with role (defaults to 'user' if not provided)
        let role = request.role.unwrap_or_default();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.name)
        .bind(role)
        .fetch_one(&self.db_pool)
        .await?;

        // Generate tokens with role
        let token_pair = generate_token_pair(&user.id, &user.email, user.role, &self.jwt_config)?;

        Ok(AuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: token_pair.token_type,
            expires_in: token_pair.expires_in,
            user: UserInfo {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user.role,
            },
        })
    }

    /// Login an existing user
    pub async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
        // Find user by email
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(&request.email)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid email or password".to_string()))?;

        // Verify password
        let is_valid = verify_password(&request.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Authentication("Invalid email or password".to_string()));
        }

        // Update last login
        sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
            .bind(&user.id)
            .execute(&self.db_pool)
            .await?;

        // Generate tokens with role
        let token_pair = generate_token_pair(&user.id, &user.email, user.role, &self.jwt_config)?;

        Ok(AuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: token_pair.token_type,
            expires_in: token_pair.expires_in,
            user: UserInfo {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user.role,
            },
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> AppResult<AuthResponse> {
        // Validate refresh token
        let claims = validate_refresh_token(&request.refresh_token, &self.jwt_config)?;

        // Get user from database
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(&user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

        // Generate new token pair with role
        let token_pair = generate_token_pair(&user.id, &user.email, user.role, &self.jwt_config)?;

        Ok(AuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: token_pair.token_type,
            expires_in: token_pair.expires_in,
            user: UserInfo {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user.role,
            },
        })
    }
}

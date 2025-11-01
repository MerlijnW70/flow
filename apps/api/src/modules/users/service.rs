use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::auth::hash::{hash_password, verify_password};
use crate::utils::error::{AppError, AppResult};

use super::model::{ChangePasswordRequest, UpdateUserRequest, User, UserResponse};

pub struct UserService {
    db_pool: PgPool,
}

impl UserService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Get user by ID
    pub async fn get_by_id(&self, user_id: &Uuid) -> AppResult<UserResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    /// Get user by email
    pub async fn get_by_email(&self, email: &str) -> AppResult<UserResponse> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    /// Update user information
    pub async fn update(
        &self,
        user_id: &Uuid,
        request: UpdateUserRequest,
    ) -> AppResult<UserResponse> {
        // Build dynamic query based on provided fields
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut has_updates = false;

        if request.name.is_some() {
            query.push_str(", name = $2");
            has_updates = true;
        }

        if !has_updates {
            return Err(AppError::BadRequest("No fields to update".to_string()));
        }

        query.push_str(" WHERE id = $1 RETURNING *");

        let mut query_builder = sqlx::query_as::<_, User>(&query).bind(user_id);

        if let Some(name) = request.name {
            query_builder = query_builder.bind(name);
        }

        let user = query_builder
            .fetch_optional(&self.db_pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: &Uuid,
        request: ChangePasswordRequest,
    ) -> AppResult<()> {
        // Get current user
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Verify current password
        let is_valid = verify_password(&request.current_password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Authentication("Current password is incorrect".to_string()));
        }

        // Hash new password
        let new_password_hash = hash_password(&request.new_password)?;

        // Update password
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(&new_password_hash)
        .bind(user_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Delete user
    pub async fn delete(&self, user_id: &Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    /// List all users (with pagination)
    pub async fn list(&self, page: u32, per_page: u32) -> AppResult<(Vec<UserResponse>, u64)> {
        let offset = (page - 1) * per_page;

        // Get total count
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db_pool)
            .await?;

        // Get paginated users
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.db_pool)
        .await?;

        let user_responses: Vec<UserResponse> = users.into_iter().map(Into::into).collect();

        Ok((user_responses, total.0 as u64))
    }
}

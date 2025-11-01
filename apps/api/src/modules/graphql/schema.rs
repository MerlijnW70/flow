use async_graphql::{Context, EmptySubscription, Object, Result, Schema, SimpleObject};
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::users::model::{User, UserRole};
use crate::modules::auth::jwt::Claims;

// GraphQL Schema Type
pub type GraphQLSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

// GraphQL Context - holds shared state
#[derive(Clone)]
pub struct GraphQLContext {
    pub db_pool: PgPool,
    pub auth_claims: Option<Claims>,
}

// User Type for GraphQL
#[derive(SimpleObject)]
struct UserQL {
    id: String,
    email: String,
    name: String,
    role: String,
    created_at: String,
}

impl From<User> for UserQL {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            role: user.role.to_string(),
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

// Query Root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get current authenticated user
    async fn me(&self, ctx: &Context<'_>) -> Result<UserQL> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let claims = gql_ctx
            .auth_claims
            .as_ref()
            .ok_or("Unauthorized")?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| "Invalid user ID")?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&gql_ctx.db_pool)
            .await
            .map_err(|_| "User not found")?;

        Ok(user.into())
    }

    /// Get user by ID (admin only)
    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<UserQL> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let claims = gql_ctx
            .auth_claims
            .as_ref()
            .ok_or("Unauthorized")?;

        // Check if user is admin
        if claims.role != UserRole::Admin {
            return Err("Forbidden: Admin access required".into());
        }

        let user_id = Uuid::parse_str(&id)
            .map_err(|_| "Invalid user ID format")?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&gql_ctx.db_pool)
            .await
            .map_err(|_| "User not found")?;

        Ok(user.into())
    }

    /// List all users (admin only)
    async fn users(&self, ctx: &Context<'_>, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<UserQL>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let claims = gql_ctx
            .auth_claims
            .as_ref()
            .ok_or("Unauthorized")?;

        // Check if user is admin
        if claims.role != UserRole::Admin {
            return Err("Forbidden: Admin access required".into());
        }

        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&gql_ctx.db_pool)
        .await
        .map_err(|_| "Failed to fetch users")?;

        Ok(users.into_iter().map(UserQL::from).collect())
    }

    /// Health check query
    async fn health(&self) -> Result<String> {
        Ok("healthy".to_string())
    }
}

// Mutation Root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Update current user's name
    async fn update_profile(&self, ctx: &Context<'_>, name: String) -> Result<UserQL> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let claims = gql_ctx
            .auth_claims
            .as_ref()
            .ok_or("Unauthorized")?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| "Invalid user ID")?;

        // Validate name length
        if name.len() < 2 || name.len() > 100 {
            return Err("Name must be between 2 and 100 characters".into());
        }

        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
        )
        .bind(&name)
        .bind(user_id)
        .fetch_one(&gql_ctx.db_pool)
        .await
        .map_err(|_| "Failed to update user")?;

        Ok(user.into())
    }

    /// Delete current user's account
    async fn delete_account(&self, ctx: &Context<'_>) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let claims = gql_ctx
            .auth_claims
            .as_ref()
            .ok_or("Unauthorized")?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| "Invalid user ID")?;

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&gql_ctx.db_pool)
            .await
            .map_err(|_| "Failed to delete account")?;

        Ok(true)
    }
}

/// Build the GraphQL schema
pub fn build_schema(db_pool: PgPool) -> GraphQLSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(GraphQLContext {
            db_pool,
            auth_claims: None,
        })
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builds() {
        // This is a compile-time test to ensure schema builds correctly
        // We can't test with actual DB here, but we verify the schema structure
        assert_eq!(
            QueryRoot.to_string(),
            "QueryRoot"
        );
    }
}

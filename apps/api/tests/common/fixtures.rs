// Test fixtures and sample data factories
// Provides consistent test data generation

use chrono::Utc;
use uuid::Uuid;
use vibe_api::modules::users::model::{User, UserRole};

/// Generate a test user with random data
pub fn create_test_user() -> User {
    let id = Uuid::new_v4();
    User {
        id,
        email: format!("user_{}@example.com", id.simple()),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test".to_string(),
        name: format!("Test User {}", &id.to_string()[..8]),
        role: UserRole::User,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: Some(Utc::now()),
    }
}

/// Generate a test user with specific fields
pub fn create_test_user_with(email: &str, name: &str) -> User {
    User {
        id: Uuid::new_v4(),
        email: email.to_string(),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test".to_string(),
        name: name.to_string(),
        role: UserRole::User,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: Some(Utc::now()),
    }
}

/// Generate multiple test users
pub fn create_test_users(count: usize) -> Vec<User> {
    (0..count).map(|_| create_test_user()).collect()
}

/// Test email address
pub const TEST_EMAIL: &str = "test@example.com";

/// Test password (plain text)
pub const TEST_PASSWORD: &str = "TestPassword123!";

/// Test user name
pub const TEST_NAME: &str = "Test User";

/// Test JWT secret
pub const TEST_JWT_SECRET: &str = "test_jwt_secret_key_for_testing_only";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_user() {
        let user = create_test_user();
        assert!(!user.email.is_empty());
        assert!(!user.name.is_empty());
        assert!(!user.id.to_string().is_empty());
    }

    #[test]
    fn test_create_test_user_with() {
        let user = create_test_user_with(TEST_EMAIL, TEST_NAME);
        assert_eq!(user.email, TEST_EMAIL);
        assert_eq!(user.name, TEST_NAME);
    }

    #[test]
    fn test_create_test_users() {
        let users = create_test_users(5);
        assert_eq!(users.len(), 5);

        // Verify all users have unique IDs
        let ids: std::collections::HashSet<_> = users.iter().map(|u| u.id).collect();
        assert_eq!(ids.len(), 5);
    }
}

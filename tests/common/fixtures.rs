/// Test data fixtures

pub const TEST_EMAIL: &str = "test@example.com";
pub const TEST_PASSWORD: &str = "SecurePassword123!";
pub const TEST_NAME: &str = "Test User";

pub const ADMIN_EMAIL: &str = "admin@example.com";
pub const ADMIN_PASSWORD: &str = "AdminPassword123!";
pub const ADMIN_NAME: &str = "Admin User";

pub const MODERATOR_EMAIL: &str = "moderator@example.com";
pub const MODERATOR_PASSWORD: &str = "ModeratorPassword123!";
pub const MODERATOR_NAME: &str = "Moderator User";

pub const USER2_EMAIL: &str = "user2@example.com";
pub const USER2_PASSWORD: &str = "UserPassword123!";
pub const USER2_NAME: &str = "Second User";

/// Generate a unique email for parallel tests
pub fn unique_email(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}+{}@test.com", prefix, timestamp)
}

/// Generate test user registration payload
pub fn user_registration_payload(email: &str, password: &str, name: &str) -> serde_json::Value {
    serde_json::json!({
        "email": email,
        "password": password,
        "name": name
    })
}

/// Generate test user registration payload with role
pub fn user_registration_with_role(
    email: &str,
    password: &str,
    name: &str,
    role: &str,
) -> serde_json::Value {
    serde_json::json!({
        "email": email,
        "password": password,
        "name": name,
        "role": role
    })
}

/// Generate login payload
pub fn login_payload(email: &str, password: &str) -> serde_json::Value {
    serde_json::json!({
        "email": email,
        "password": password
    })
}

/// Generate update user payload
pub fn update_user_payload(name: &str) -> serde_json::Value {
    serde_json::json!({
        "name": name
    })
}

/// Generate change password payload
pub fn change_password_payload(current: &str, new: &str) -> serde_json::Value {
    serde_json::json!({
        "current_password": current,
        "new_password": new
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_email_generation() {
        let email1 = unique_email("test");
        let email2 = unique_email("test");
        assert_ne!(email1, email2);
        assert!(email1.starts_with("test+"));
        assert!(email1.ends_with("@test.com"));
    }

    #[test]
    fn test_user_registration_payload() {
        let payload = user_registration_payload(TEST_EMAIL, TEST_PASSWORD, TEST_NAME);
        assert_eq!(payload["email"], TEST_EMAIL);
        assert_eq!(payload["password"], TEST_PASSWORD);
        assert_eq!(payload["name"], TEST_NAME);
    }

    #[test]
    fn test_user_registration_with_role() {
        let payload = user_registration_with_role(ADMIN_EMAIL, ADMIN_PASSWORD, ADMIN_NAME, "admin");
        assert_eq!(payload["email"], ADMIN_EMAIL);
        assert_eq!(payload["role"], "admin");
    }
}

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::utils::error::{AppError, AppResult};

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalServer(format!("Failed to hash password: {}", e)))
        .map(|hash| hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::InternalServer(format!("Failed to parse hash: {}", e)))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).expect("Failed to hash password");

        assert!(verify_password(password, &hash).expect("Failed to verify password"));
        assert!(!verify_password("WrongPassword", &hash).expect("Failed to verify password"));
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let password = "Password123!";
        let hash1 = hash_password(password).expect("Failed to hash password");
        let hash2 = hash_password(password).expect("Failed to hash password");

        // Same password should produce different hashes due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).expect("Failed to verify"));
        assert!(verify_password(password, &hash2).expect("Failed to verify"));
    }
}

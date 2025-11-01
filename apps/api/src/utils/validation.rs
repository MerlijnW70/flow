use validator::{Validate, ValidationError};
use crate::utils::error::{AppError, AppResult};

/// Validate a struct and convert validation errors to AppError
pub fn validate_struct<T: Validate>(data: &T) -> AppResult<()> {
    data.validate()
        .map_err(|e| AppError::Validation(format!("{}", e)))
}

/// Custom email validator (can be used with validator crate)
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = regex::Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    if email_regex.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_email"))
    }
}

/// Custom password strength validator
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("password_too_short"));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !(has_uppercase && has_lowercase && has_digit && has_special) {
        return Err(ValidationError::new("password_too_weak"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@example.co.uk").is_ok());
        assert!(validate_email("invalid.email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(validate_password_strength("StrongP@ss123").is_ok());
        assert!(validate_password_strength("weak").is_err());
        assert!(validate_password_strength("NoSpecial123").is_err());
        assert!(validate_password_strength("nouppercas3!").is_err());
    }
}

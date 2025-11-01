// Configuration & Environment Tests
// Validates that configuration loading works correctly

mod common;

use common::TEST_CONFIG;

#[test]
fn test_database_url_is_set() {
    let config = &*TEST_CONFIG;
    assert!(
        !config.database_url.is_empty(),
        "DATABASE_URL must be set"
    );
    assert!(
        config.database_url.starts_with("postgres://"),
        "DATABASE_URL must be a PostgreSQL connection string"
    );
}

#[test]
fn test_jwt_secret_is_set() {
    let config = &*TEST_CONFIG;
    assert!(
        !config.jwt_secret.is_empty(),
        "JWT_SECRET must be set"
    );
    assert!(
        config.jwt_secret.len() >= 16,
        "JWT_SECRET should be at least 16 characters for security"
    );
}

#[test]
fn test_server_port_is_valid() {
    let config = &*TEST_CONFIG;
    assert!(
        config.server_port > 0,
        "SERVER_PORT must be greater than 0"
    );
    assert!(
        config.server_port < 65536,
        "SERVER_PORT must be less than 65536"
    );
}

#[test]
fn test_database_url_contains_test_db() {
    let config = &*TEST_CONFIG;
    // Ensure we're using a test database to avoid accidents
    assert!(
        config.database_url.contains("test") || config.database_url.contains("_test"),
        "DATABASE_URL should contain 'test' to avoid using production database"
    );
}

#[test]
fn test_config_can_be_cloned() {
    let config = TEST_CONFIG.clone();
    assert_eq!(config.database_url, TEST_CONFIG.database_url);
    assert_eq!(config.jwt_secret, TEST_CONFIG.jwt_secret);
    assert_eq!(config.server_port, TEST_CONFIG.server_port);
}

#[test]
fn test_env_file_loading() {
    // Verify that .env.test can be loaded
    let result = dotenvy::from_filename(".env.test");
    // It's okay if the file doesn't exist in CI, but if it does, it should load
    if result.is_ok() {
        // If .env.test exists, verify we can read from it
        assert!(std::env::var("ENVIRONMENT").is_ok() || std::env::var("SERVER_PORT").is_ok());
    }
}

#[test]
fn test_jwt_config_creation() {
    use common::create_test_jwt_config;

    let jwt_config = create_test_jwt_config();

    assert!(!jwt_config.secret.is_empty());
    assert_eq!(jwt_config.access_token_expiry_hours, 1);
    assert_eq!(jwt_config.refresh_token_expiry_days, 7);
    assert_eq!(jwt_config.issuer, "vibe-api-test");
}

#[test]
fn test_multiple_config_accesses() {
    // Verify that lazy static works correctly with multiple accesses
    let config1 = &*TEST_CONFIG;
    let config2 = &*TEST_CONFIG;

    assert_eq!(config1.database_url, config2.database_url);
    assert_eq!(config1.jwt_secret, config2.jwt_secret);
    assert_eq!(config1.server_port, config2.server_port);
}

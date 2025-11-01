// Unit tests for configuration module

#[cfg(test)]
mod tests {
    use super::super::*;
    use temp_env::with_vars;

    #[test]
    fn test_parse_environment_development() {
        assert_eq!(
            Config::parse_environment("development"),
            Environment::Development
        );
        assert_eq!(
            Config::parse_environment("dev"),
            Environment::Development
        );
    }

    #[test]
    fn test_parse_environment_production() {
        assert_eq!(
            Config::parse_environment("production"),
            Environment::Production
        );
        assert_eq!(
            Config::parse_environment("prod"),
            Environment::Production
        );
    }

    #[test]
    fn test_parse_environment_test() {
        assert_eq!(Config::parse_environment("test"), Environment::Test);
    }

    #[test]
    fn test_parse_cors_origins_any() {
        let origins = Config::parse_cors_origins("*");
        // Just verify it doesn't panic
        drop(origins);
    }

    #[test]
    fn test_parse_cors_origins_list() {
        let origins = Config::parse_cors_origins("http://localhost:3000,http://localhost:3001");
        // Just verify it doesn't panic
        drop(origins);
    }

    #[test]
    fn test_config_load_with_env_vars() {
        with_vars(
            vec![
                ("PORT", Some("8080")),
                ("HOST", Some("localhost")),
                ("ENVIRONMENT", Some("test")),
                ("CORS_ORIGINS", Some("http://localhost:3000")),
                (
                    "DATABASE_URL",
                    Some("postgresql://test:test@localhost/test"),
                ),
                ("JWT_SECRET", Some("test_secret")),
                ("JWT_ISSUER", Some("test-issuer")),
            ],
            || {
                let config = Config::load();
                assert!(config.is_ok(), "Config loading should succeed");

                let config = config.unwrap();
                assert_eq!(config.server.port, 8080);
                assert_eq!(config.server.host, "localhost");
                assert_eq!(config.jwt.issuer, "test-issuer");
            },
        );
    }

    #[test]
    fn test_config_load_missing_database_url() {
        with_vars(
            vec![
                ("DATABASE_URL", None::<&str>),
                ("JWT_SECRET", Some("test_secret")),
            ],
            || {
                let result = std::panic::catch_unwind(|| {
                    let _ = Config::load();
                });
                assert!(result.is_err(), "Should panic without DATABASE_URL");
            },
        );
    }

    #[test]
    fn test_config_load_missing_jwt_secret() {
        with_vars(
            vec![
                ("DATABASE_URL", Some("postgresql://test:test@localhost/test")),
                ("JWT_SECRET", None::<&str>),
            ],
            || {
                let result = std::panic::catch_unwind(|| {
                    let _ = Config::load();
                });
                assert!(result.is_err(), "Should panic without JWT_SECRET");
            },
        );
    }

    #[test]
    fn test_config_defaults() {
        with_vars(
            vec![
                ("DATABASE_URL", Some("postgresql://test:test@localhost/test")),
                ("JWT_SECRET", Some("secret")),
                // Don't set optional values
                ("PORT", None::<&str>),
                ("DB_MAX_CONNECTIONS", None::<&str>),
            ],
            || {
                let config = Config::load().unwrap();
                assert_eq!(config.server.port, 3000); // Default port
                assert_eq!(config.database.max_connections, 10); // Default max connections
            },
        );
    }
}

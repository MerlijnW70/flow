use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    #[cfg(feature = "ai")]
    pub ai: AiConfig,
    #[cfg(feature = "storage")]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub cors_origins: Vec<String>,
    pub environment: Environment,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub issuer: String,
}

#[cfg(feature = "ai")]
#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub default_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[cfg(feature = "storage")]
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_endpoint: Option<String>,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub max_file_size_mb: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        // Load .env file
        dotenvy::dotenv().ok();

        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let server = ServerConfig {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            host: env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            cors_origins: Self::parse_cors_origins(
                &env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string())
            ),
            environment: Self::parse_environment(&environment),
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("DB_MAX_CONNECTIONS must be a valid number"),
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .expect("DB_MIN_CONNECTIONS must be a valid number"),
            acquire_timeout_secs: env::var("DB_ACQUIRE_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("DB_ACQUIRE_TIMEOUT_SECS must be a valid number"),
            idle_timeout_secs: env::var("DB_IDLE_TIMEOUT_SECS")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .expect("DB_IDLE_TIMEOUT_SECS must be a valid number"),
        };

        let jwt = JwtConfig {
            secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            access_token_expiry_hours: env::var("JWT_ACCESS_TOKEN_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_ACCESS_TOKEN_EXPIRY_HOURS must be a valid number"),
            refresh_token_expiry_days: env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("JWT_REFRESH_TOKEN_EXPIRY_DAYS must be a valid number"),
            issuer: env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "vibe-api".to_string()),
        };

        #[cfg(feature = "ai")]
        let ai = AiConfig {
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            default_model: env::var("AI_DEFAULT_MODEL")
                .unwrap_or_else(|_| "gpt-4".to_string()),
            max_tokens: env::var("AI_MAX_TOKENS")
                .unwrap_or_else(|_| "2000".to_string())
                .parse()
                .expect("AI_MAX_TOKENS must be a valid number"),
            temperature: env::var("AI_TEMPERATURE")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()
                .expect("AI_TEMPERATURE must be a valid float"),
        };

        #[cfg(feature = "storage")]
        let storage = StorageConfig {
            s3_bucket: env::var("S3_BUCKET")
                .expect("S3_BUCKET must be set when storage feature is enabled"),
            s3_region: env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
            s3_access_key: env::var("S3_ACCESS_KEY")
                .expect("S3_ACCESS_KEY must be set when storage feature is enabled"),
            s3_secret_key: env::var("S3_SECRET_KEY")
                .expect("S3_SECRET_KEY must be set when storage feature is enabled"),
            max_file_size_mb: env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("MAX_FILE_SIZE_MB must be a valid number"),
        };

        Ok(Config {
            server,
            database,
            jwt,
            #[cfg(feature = "ai")]
            ai,
            #[cfg(feature = "storage")]
            storage,
        })
    }

    fn parse_environment(env_str: &str) -> Environment {
        match env_str.to_lowercase().as_str() {
            "production" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        }
    }

    fn parse_cors_origins(origins: &str) -> Vec<String> {
        origins
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests;


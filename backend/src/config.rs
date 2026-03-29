use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub app_base_url: String,
    pub issuer: String,
    pub session_secret: String,
    pub cookie_domain: Option<String>,
    pub access_token_ttl: i64,
    pub refresh_token_ttl: i64,
    pub session_ttl: i64,
    pub storage_path: String,
    pub private_key_encryption_key: String,
    pub rate_limit_max_requests: u32,
    pub rate_limit_window_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: "postgres://localhost/oidc_provider".to_string(),
            app_base_url: "http://localhost:3000".to_string(),
            issuer: "http://localhost:3000".to_string(),
            session_secret: "dev-secret-key-change-in-production".to_string(),
            cookie_domain: None,
            access_token_ttl: 3600,
            refresh_token_ttl: 604800,
            session_ttl: 86400,
            storage_path: "./storage".to_string(),
            private_key_encryption_key: "dev-encryption-key-32-chars-change!!".to_string(),
            rate_limit_max_requests: 100,
            rate_limit_window_secs: 60,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/oidc_provider".to_string()),
            app_base_url: env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            issuer: env::var("ISSUER")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string()),
            cookie_domain: env::var("COOKIE_DOMAIN").ok(),
            access_token_ttl: env::var("ACCESS_TOKEN_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()?,
            refresh_token_ttl: env::var("REFRESH_TOKEN_TTL")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()?,
            session_ttl: env::var("SESSION_TTL")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()?,
            storage_path: env::var("STORAGE_PATH")
                .unwrap_or_else(|_| "./storage".to_string()),
            private_key_encryption_key: env::var("PRIVATE_KEY_ENCRYPTION_KEY")
                .unwrap_or_else(|_| "dev-encryption-key-32-chars-change!!".to_string()),
            rate_limit_max_requests: env::var("RATE_LIMIT_MAX_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            rate_limit_window_secs: env::var("RATE_LIMIT_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
        })
    }
}

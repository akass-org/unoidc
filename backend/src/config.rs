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
    pub rate_limit_login_max_requests: u32,
    pub rate_limit_login_window_secs: u64,
    pub rate_limit_token_max_requests: u32,
    pub rate_limit_token_window_secs: u64,
    pub cors_allowed_origins: Vec<String>,
    pub trusted_proxy_ips: Vec<String>,
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
            rate_limit_login_max_requests: 10,
            rate_limit_login_window_secs: 60,
            rate_limit_token_max_requests: 30,
            rate_limit_token_window_secs: 60,
            cors_allowed_origins: vec![
                "http://localhost:5173".to_string(),
                "http://localhost:3000".to_string(),
            ],
            trusted_proxy_ips: vec![],
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let config = Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/oidc_provider".to_string()),
            app_base_url: env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            issuer: env::var("ISSUER").unwrap_or_else(|_| "http://localhost:3000".to_string()),
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
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "./storage".to_string()),
            private_key_encryption_key: env::var("PRIVATE_KEY_ENCRYPTION_KEY")
                .unwrap_or_else(|_| "dev-encryption-key-32-chars-change!!".to_string()),
            rate_limit_max_requests: env::var("RATE_LIMIT_MAX_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            rate_limit_window_secs: env::var("RATE_LIMIT_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
            rate_limit_login_max_requests: env::var("RATE_LIMIT_LOGIN_MAX_REQUESTS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            rate_limit_login_window_secs: env::var("RATE_LIMIT_LOGIN_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
            rate_limit_token_max_requests: env::var("RATE_LIMIT_TOKEN_MAX_REQUESTS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            rate_limit_token_window_secs: env::var("RATE_LIMIT_TOKEN_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            trusted_proxy_ips: env::var("TRUSTED_PROXY_IPS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        };

        // 生产环境配置验证
        config.validate_production()?;

        Ok(config)
    }

    /// 生产环境配置验证
    ///
    /// 检查是否使用了不安全的默认值
    pub fn validate_production(&self) -> Result<(), anyhow::Error> {
        if self.private_key_encryption_key.len() < 16 {
            return Err(anyhow::anyhow!(
                "PRIVATE_KEY_ENCRYPTION_KEY must be at least 16 characters (got {})",
                self.private_key_encryption_key.len()
            ));
        }

        if self.private_key_encryption_key.len() < 32 {
            tracing::warn!(
                "PRIVATE_KEY_ENCRYPTION_KEY is shorter than recommended 32 characters (got {})",
                self.private_key_encryption_key.len()
            );
        }

        if self.session_secret == "dev-secret-key-change-in-production" {
            tracing::warn!("SESSION_SECRET is using default value - MUST change in production");
        }

        if self.session_secret.len() < 32 {
            tracing::warn!(
                "SESSION_SECRET is shorter than recommended 32 characters (got {})",
                self.session_secret.len()
            );
        }

        let is_production = !self.issuer.contains("localhost")
            && !self.issuer.contains("127.0.0.1")
            && !self.issuer.starts_with("http://");

        if !is_production {
            return Ok(());
        }

        let mut warnings = Vec::new();

        if self.session_secret == "dev-secret-key-change-in-production" {
            warnings.push("SESSION_SECRET is using default value - MUST change in production");
        }

        if self.session_secret.len() < 32 {
            warnings.push("SESSION_SECRET should be at least 32 characters");
        }

        if self.private_key_encryption_key == "dev-encryption-key-32-chars-change!!" {
            warnings.push(
                "PRIVATE_KEY_ENCRYPTION_KEY is using default value - MUST change in production",
            );
        }

        if self.issuer.starts_with("http://") {
            warnings.push("ISSUER should use HTTPS in production");
        }

        if self.app_base_url.starts_with("http://") {
            warnings.push("APP_BASE_URL should use HTTPS in production");
        }

        if !warnings.is_empty() {
            tracing::error!("Production configuration validation failed:");
            for warning in &warnings {
                tracing::error!("  - {}", warning);
            }
            return Err(anyhow::anyhow!(
                "Production configuration validation failed: {}",
                warnings.join("; ")
            ));
        }

        tracing::info!("Production configuration validated successfully");
        Ok(())
    }
}

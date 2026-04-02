// Rate limit service
//
// Provides distributed rate limiting capabilities

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Rate limit key types
#[derive(Debug, Clone)]
pub enum RateLimitKey {
    /// User ID
    UserId(Uuid),
    /// IP address
    IpAddress(String),
    /// Client ID
    ClientId(Uuid),
    /// Custom identifier
    Custom(String),
}

impl RateLimitKey {
    pub fn as_string(&self) -> String {
        match self {
            RateLimitKey::UserId(id) => format!("uid:{}", id),
            RateLimitKey::IpAddress(addr) => format!("ip:{}", addr),
            RateLimitKey::ClientId(id) => format!("client:{}", id),
            RateLimitKey::Custom(s) => format!("custom:{}", s),
        }
    }
}

/// Rate limit tier configurations
#[derive(Debug, Clone)]
pub struct RateLimitTier {
    pub name: &'static str,
    pub max_requests: u32,
    pub window_secs: u64,
}

impl RateLimitTier {
    pub fn global() -> Self {
        Self {
            name: "global",
            max_requests: 1000,
            window_secs: 60,
        }
    }

    pub fn login() -> Self {
        Self {
            name: "login",
            max_requests: 10,
            window_secs: 60,
        }
    }

    pub fn token() -> Self {
        Self {
            name: "token",
            max_requests: 30,
            window_secs: 60,
        }
    }

    pub fn api() -> Self {
        Self {
            name: "api",
            max_requests: 100,
            window_secs: 60,
        }
    }
}

pub struct RateLimitService;

impl RateLimitService {
    /// Check if a request is allowed under the rate limit
    ///
    /// Returns (allowed, remaining_requests, reset_time_secs)
    pub async fn check_rate_limit(
        _pool: &PgPool,
        key: &RateLimitKey,
        tier: &RateLimitTier,
    ) -> Result<(bool, u32, u64)> {
        let key_str = key.as_string();
        
        // TODO: Use Redis or other cache for distributed rate limiting
        // For now, this is a placeholder for single-instance in-memory implementation via middleware
        // Production should use Redis for distributed deployments

        // In single-instance deployment, the middleware layer handles this
        // This service is here for future Redis integration

        tracing::debug!(
            "Rate limit check: key={}, tier={}, max={}, window={}s",
            key_str, tier.name, tier.max_requests, tier.window_secs
        );

        let allowed = true; // Middleware handles actual enforcement
        let remaining = tier.max_requests;
        let reset_secs = tier.window_secs;

        Ok((allowed, remaining, reset_secs))
    }

    /// Reset rate limit for a key (admin operation)
    pub async fn reset_limit(
        _pool: &PgPool,
        key: &RateLimitKey,
    ) -> Result<()> {
        let key_str = key.as_string();
        tracing::info!("Rate limit reset for key: {}", key_str);
        // TODO: Clear from Redis when Redis integration is added
        Ok(())
    }

    /// Get current rate limit status
    pub async fn get_status(
        pool: &PgPool,
        key: &RateLimitKey,
        tier: &RateLimitTier,
    ) -> Result<RateLimitStatus> {
        let (allowed, remaining, reset_secs) = Self::check_rate_limit(pool, key, tier).await?;

        Ok(RateLimitStatus {
            allowed,
            key: key.as_string(),
            tier_name: tier.name.to_string(),
            max_requests: tier.max_requests,
            remaining,
            reset_in_secs: reset_secs,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RateLimitStatus {
    pub allowed: bool,
    pub key: String,
    pub tier_name: String,
    pub max_requests: u32,
    pub remaining: u32,
    pub reset_in_secs: u64,
}

// Rate limit service
//
// 基于 PostgreSQL 的固定窗口限流实现

use anyhow::Result;
use sqlx::PgPool;
use time::OffsetDateTime;
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
    /// 检查是否在限额内（固定窗口算法）
    ///
    /// 使用 INSERT ... ON CONFLICT 原子地递增计数器
    /// Returns (allowed, remaining_requests, reset_time_secs)
    pub async fn check_rate_limit(
        pool: &PgPool,
        key: &RateLimitKey,
        tier: &RateLimitTier,
    ) -> Result<(bool, u32, u64)> {
        let key_str = key.as_string();
        let now = OffsetDateTime::now_utc();

        // 计算当前窗口起始时间（对齐到窗口边界）
        let unix_now = now.unix_timestamp();
        let window_start_unix = unix_now - (unix_now % tier.window_secs as i64);
        let window_start = OffsetDateTime::from_unix_timestamp(window_start_unix)?;

        // 原子操作：插入或更新，返回当前请求数
        let result: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO rate_limits (key, window_start, request_count)
            VALUES ($1, $2, 1)
            ON CONFLICT (key) DO UPDATE SET
                request_count = CASE
                    WHEN rate_limits.window_start = $2 THEN rate_limits.request_count + 1
                    ELSE 1
                END,
                window_start = $2
            RETURNING request_count
            "#,
        )
        .bind(&key_str)
        .bind(window_start)
        .fetch_one(pool)
        .await?;

        let count = result.0 as u32;
        let allowed = count <= tier.max_requests;
        let remaining = if count >= tier.max_requests {
            0
        } else {
            tier.max_requests - count
        };
        let elapsed = (now.unix_timestamp() - window_start_unix).max(0) as u64;
        let reset_secs = tier.window_secs.saturating_sub(elapsed);

        tracing::debug!(
            key = %key_str,
            tier = tier.name,
            count = count,
            max = tier.max_requests,
            allowed = allowed,
            "Rate limit check"
        );

        Ok((allowed, remaining, reset_secs))
    }

    /// 重置某个 key 的限流（管理操作）
    pub async fn reset_limit(pool: &PgPool, key: &RateLimitKey) -> Result<()> {
        let key_str = key.as_string();
        sqlx::query("DELETE FROM rate_limits WHERE key = $1")
            .bind(&key_str)
            .execute(pool)
            .await?;
        tracing::info!("Rate limit reset for key: {}", key_str);
        Ok(())
    }

    /// 清理过期的限流记录（应由后台任务调用）
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64> {
        let result =
            sqlx::query("DELETE FROM rate_limits WHERE window_start < NOW() - INTERVAL '1 hour'")
                .execute(pool)
                .await?;
        Ok(result.rows_affected())
    }

    /// 获取当前限流状态
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_key_formats() {
        let uid = Uuid::new_v4();
        assert_eq!(
            RateLimitKey::UserId(uid).as_string(),
            format!("uid:{}", uid)
        );
        assert_eq!(
            RateLimitKey::IpAddress("1.2.3.4".to_string()).as_string(),
            "ip:1.2.3.4"
        );
        let cid = Uuid::new_v4();
        assert_eq!(
            RateLimitKey::ClientId(cid).as_string(),
            format!("client:{}", cid)
        );
        assert_eq!(
            RateLimitKey::Custom("my-key".to_string()).as_string(),
            "custom:my-key"
        );
    }

    #[test]
    fn test_tier_defaults() {
        let g = RateLimitTier::global();
        assert_eq!(g.name, "global");
        assert_eq!(g.max_requests, 1000);
        assert_eq!(g.window_secs, 60);

        let l = RateLimitTier::login();
        assert_eq!(l.name, "login");
        assert_eq!(l.max_requests, 10);

        let t = RateLimitTier::token();
        assert_eq!(t.name, "token");
        assert_eq!(t.max_requests, 30);
    }
}

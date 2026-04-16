use axum::{
    extract::ConnectInfo,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub window_secs: u64,
    pub max_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            window_secs: 60,
            max_requests: 100,
        }
    }
}

#[derive(Debug, Clone)]
struct RequestRecord {
    timestamp: Instant,
    count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitTier {
    Global,
    Login,
    Token,
}

impl RateLimitTier {
    pub fn from_path(path: &str) -> Self {
        if path.starts_with("/api/v1/auth/login") || path.starts_with("/api/v1/auth/register") {
            Self::Login
        } else if path.starts_with("/token") {
            Self::Token
        } else {
            Self::Global
        }
    }
}

#[derive(Debug)]
struct TierLimiter {
    records: RwLock<HashMap<String, RequestRecord>>,
    config: RateLimitConfig,
}

impl TierLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            config,
        }
    }

    async fn check(&self, key: &str) -> std::result::Result<bool, u64> {
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_secs);

        let mut records = self.records.write().await;

        records.retain(|_, record| now.duration_since(record.timestamp) < window);

        match records.get_mut(key) {
            Some(record) => {
                let elapsed = now.duration_since(record.timestamp);
                if elapsed < window {
                    if record.count >= self.config.max_requests {
                        let retry_after = window - elapsed;
                        Err(retry_after.as_secs())
                    } else {
                        record.count += 1;
                        Ok(true)
                    }
                } else {
                    record.timestamp = now;
                    record.count = 1;
                    Ok(true)
                }
            }
            None => {
                records.insert(
                    key.to_string(),
                    RequestRecord {
                        timestamp: now,
                        count: 1,
                    },
                );
                Ok(true)
            }
        }
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    tiers: HashMap<RateLimitTier, TierLimiter>,
    trusted_proxy_ips: Vec<String>,
}

impl RateLimiter {
    pub fn new(
        global_config: RateLimitConfig,
        login_config: RateLimitConfig,
        token_config: RateLimitConfig,
        trusted_proxy_ips: Vec<String>,
    ) -> Self {
        let mut tiers = HashMap::new();
        tiers.insert(RateLimitTier::Global, TierLimiter::new(global_config));
        tiers.insert(RateLimitTier::Login, TierLimiter::new(login_config));
        tiers.insert(RateLimitTier::Token, TierLimiter::new(token_config));
        Self {
            tiers,
            trusted_proxy_ips,
        }
    }

    pub async fn check(&self, tier: RateLimitTier, ip: &str) -> std::result::Result<bool, u64> {
        if let Some(limiter) = self.tiers.get(&tier) {
            limiter.check(ip).await
        } else {
            Ok(true)
        }
    }
}

pub fn extract_client_ip(
    headers: &axum::http::HeaderMap,
    remote_addr: Option<&str>,
    trusted_proxy_ips: &[String],
) -> String {
    let remote_ip = remote_addr
        .and_then(|addr| addr.parse::<std::net::SocketAddr>().ok())
        .map(|sa| sa.ip().to_string());

    let is_trusted_proxy = remote_ip
        .as_ref()
        .map(|ip| trusted_proxy_ips.contains(ip))
        .unwrap_or(false);

    if is_trusted_proxy {
        if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(first_ip) = xff.split(',').next() {
                let ip = first_ip.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }

        if let Some(real_ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            let ip = real_ip.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    remote_ip.unwrap_or_else(|| "unknown".to_string())
}

pub async fn rate_limit_middleware(
    axum::Extension(limiter): axum::Extension<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    // 优先读取测试注入的 Option<String>，否则回退到真实 ConnectInfo<SocketAddr>
    let remote_addr_from_ext = request
        .extensions()
        .get::<Option<String>>()
        .and_then(|v| v.clone());
    let remote_addr = remote_addr_from_ext.or_else(|| {
        request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.to_string())
    });

    let ip = extract_client_ip(
        request.headers(),
        remote_addr.as_deref(),
        &limiter.trusted_proxy_ips,
    );

    let tier = RateLimitTier::from_path(&path);

    match limiter.check(tier, &ip).await {
        Ok(true) => next.run(request).await,
        Err(retry_after) => {
            tracing::warn!(
                ip = %ip,
                path = %path,
                tier = ?tier,
                retry_after = retry_after,
                "Rate limit exceeded"
            );
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "error_code": "RATE_LIMIT_EXCEEDED",
                    "status": 429,
                    "details": format!("Retry after {} seconds", retry_after),
                })),
            )
                .into_response();

            if let Ok(val) = HeaderValue::from_str(&retry_after.to_string()) {
                response
                    .headers_mut()
                    .insert(axum::http::header::RETRY_AFTER, val);
            }

            response
        }
        _ => next.run(request).await,
    }
}

pub fn create_rate_limiter(
    global_max: u32,
    global_window: u64,
    login_max: u32,
    login_window: u64,
    token_max: u32,
    token_window: u64,
    trusted_proxy_ips: Vec<String>,
) -> Arc<RateLimiter> {
    Arc::new(RateLimiter::new(
        RateLimitConfig {
            window_secs: global_window,
            max_requests: global_max,
        },
        RateLimitConfig {
            window_secs: login_window,
            max_requests: login_max,
        },
        RateLimitConfig {
            window_secs: token_window,
            max_requests: token_max,
        },
        trusted_proxy_ips,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limiter() -> RateLimiter {
        RateLimiter::new(
            RateLimitConfig {
                window_secs: 60,
                max_requests: 100,
            },
            RateLimitConfig {
                window_secs: 60,
                max_requests: 3,
            },
            RateLimitConfig {
                window_secs: 60,
                max_requests: 5,
            },
            vec!["10.0.0.1".to_string()],
        )
    }

    #[tokio::test]
    async fn test_allows_within_limit() {
        let limiter = make_limiter();
        for _ in 0..100 {
            assert!(limiter
                .check(RateLimitTier::Global, "192.168.1.1")
                .await
                .is_ok());
        }
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_different_ips_independent() {
        let limiter = make_limiter();
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_ok());
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_ok());
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.2")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_window_reset() {
        let limiter = RateLimiter::new(
            RateLimitConfig {
                window_secs: 1,
                max_requests: 2,
            },
            RateLimitConfig {
                window_secs: 1,
                max_requests: 2,
            },
            RateLimitConfig {
                window_secs: 1,
                max_requests: 2,
            },
            vec![],
        );

        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_ok());
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_ok());
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_err());

        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(limiter
            .check(RateLimitTier::Global, "192.168.1.1")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_per_tier_limits() {
        let limiter = make_limiter();

        for _ in 0..3 {
            assert!(limiter
                .check(RateLimitTier::Login, "10.0.0.1")
                .await
                .is_ok());
        }
        assert!(limiter
            .check(RateLimitTier::Login, "10.0.0.1")
            .await
            .is_err());
        assert!(limiter
            .check(RateLimitTier::Token, "10.0.0.1")
            .await
            .is_ok());
        assert!(limiter
            .check(RateLimitTier::Global, "10.0.0.1")
            .await
            .is_ok());
    }

    #[test]
    fn test_tier_from_path() {
        assert_eq!(
            RateLimitTier::from_path("/api/v1/auth/login"),
            RateLimitTier::Login
        );
        assert_eq!(
            RateLimitTier::from_path("/api/v1/auth/register"),
            RateLimitTier::Login
        );
        assert_eq!(RateLimitTier::from_path("/token"), RateLimitTier::Token);
        assert_eq!(
            RateLimitTier::from_path("/authorize"),
            RateLimitTier::Global
        );
        assert_eq!(RateLimitTier::from_path("/userinfo"), RateLimitTier::Global);
        assert_eq!(
            RateLimitTier::from_path("/.well-known/openid-configuration"),
            RateLimitTier::Global,
        );
    }

    #[test]
    fn test_extract_ip_xff_trusted() {
        let mut h = axum::http::HeaderMap::new();
        h.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        let trusted = vec!["10.0.0.1".to_string()];
        assert_eq!(
            extract_client_ip(&h, Some("10.0.0.1:12345"), &trusted),
            "1.2.3.4"
        );
    }

    #[test]
    fn test_extract_ip_xff_untrusted() {
        let mut h = axum::http::HeaderMap::new();
        h.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        let trusted: Vec<String> = vec![];
        assert_eq!(
            extract_client_ip(&h, Some("10.0.0.1:12345"), &trusted),
            "10.0.0.1"
        );
    }

    #[test]
    fn test_extract_ip_real_ip_trusted() {
        let mut h = axum::http::HeaderMap::new();
        h.insert("x-real-ip", "1.2.3.4".parse().unwrap());
        let trusted = vec!["10.0.0.1".to_string()];
        assert_eq!(
            extract_client_ip(&h, Some("10.0.0.1:12345"), &trusted),
            "1.2.3.4"
        );
    }

    #[test]
    fn test_extract_ip_fallback_remote() {
        let h = axum::http::HeaderMap::new();
        let trusted: Vec<String> = vec![];
        assert_eq!(
            extract_client_ip(&h, Some("10.0.0.1:12345"), &trusted),
            "10.0.0.1"
        );
    }

    #[test]
    fn test_extract_ip_unknown() {
        let h = axum::http::HeaderMap::new();
        let trusted: Vec<String> = vec![];
        assert_eq!(extract_client_ip(&h, None, &trusted), "unknown");
    }
}

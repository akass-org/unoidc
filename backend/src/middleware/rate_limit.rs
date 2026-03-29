// 速率限制中间件
//
// 基于滑动窗口算法的 IP 限流

use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
    Extension,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

use crate::error::{AppError, Result};

/// 限流配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 时间窗口（秒）
    pub window_secs: u64,
    /// 窗口内最大请求数
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

/// 请求记录
#[derive(Debug, Clone)]
struct RequestRecord {
    /// 请求时间戳
    timestamp: Instant,
    /// 计数
    count: u32,
}

/// 限流状态
#[derive(Debug, Default)]
pub struct RateLimiter {
    /// IP -> 请求记录
    records: RwLock<HashMap<String, RequestRecord>>,
    /// 配置
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// 检查是否允许请求
    ///
    /// 返回 Ok(true) 表示允许，Err 包含重试时间
    pub async fn check(&self, ip: &str) -> std::result::Result<bool, u64> {
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_secs);

        let mut records = self.records.write().await;

        // 清理过期记录
        records.retain(|_, record| now.duration_since(record.timestamp) < window);

        // 检查当前 IP
        match records.get_mut(ip) {
            Some(record) => {
                let elapsed = now.duration_since(record.timestamp);

                if elapsed < window {
                    // 在窗口内
                    if record.count >= self.config.max_requests {
                        // 超过限制，计算重试时间
                        let retry_after = window - elapsed;
                        Err(retry_after.as_secs())
                    } else {
                        record.count += 1;
                        Ok(true)
                    }
                } else {
                    // 窗口过期，重置
                    record.timestamp = now;
                    record.count = 1;
                    Ok(true)
                }
            }
            None => {
                // 新 IP
                records.insert(
                    ip.to_string(),
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

/// 限流中间件
///
/// 对每个 IP 进行速率限制
pub async fn rate_limit_middleware(
    Extension(limiter): Extension<Arc<RateLimiter>>,
    Extension(addr): Extension<Option<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>> {
    // 获取客户端 IP
    let ip = addr
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 检查限流
    match limiter.check(&ip).await {
        Ok(true) => Ok(next.run(request).await),
        Err(retry_after) => {
            tracing::warn!(
                "Rate limit exceeded for IP: {}, retry after {}s",
                ip,
                retry_after
            );
            Err(AppError::RateLimitExceeded {
                retry_after: Some(retry_after),
            })
        }
        _ => Ok(next.run(request).await),
    }
}

/// 创建限流器实例
pub fn create_rate_limiter(max_requests: u32, window_secs: u64) -> Arc<RateLimiter> {
    Arc::new(RateLimiter::new(RateLimitConfig {
        window_secs,
        max_requests,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            window_secs: 60,
            max_requests: 5,
        });

        // 前 5 次应该都允许
        for _ in 0..5 {
            assert!(limiter.check("192.168.1.1").await.is_ok());
        }

        // 第 6 次应该被拒绝
        let result = limiter.check("192.168.1.1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(RateLimitConfig {
            window_secs: 60,
            max_requests: 2,
        });

        // IP1 请求 2 次
        assert!(limiter.check("192.168.1.1").await.is_ok());
        assert!(limiter.check("192.168.1.1").await.is_ok());

        // IP1 被限流
        assert!(limiter.check("192.168.1.1").await.is_err());

        // IP2 仍然可以请求
        assert!(limiter.check("192.168.1.2").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let limiter = RateLimiter::new(RateLimitConfig {
            window_secs: 1, // 1 秒窗口
            max_requests: 2,
        });

        // 用完配额
        assert!(limiter.check("192.168.1.1").await.is_ok());
        assert!(limiter.check("192.168.1.1").await.is_ok());
        assert!(limiter.check("192.168.1.1").await.is_err());

        // 等待窗口过期
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // 应该可以再次请求
        assert!(limiter.check("192.168.1.1").await.is_ok());
    }
}

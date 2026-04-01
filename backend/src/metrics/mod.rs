// Prometheus Metrics
//
// 定义和初始化 OIDC Provider 的 Prometheus 指标

use lazy_static::lazy_static;
use prometheus::{Counter, Gauge, Histogram, Registry};
use std::sync::OnceLock;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // ===== 认证相关指标 =====

    /// 授权请求总数
    pub static ref AUTH_REQUESTS_TOTAL: Counter = Counter::new(
        "oidc_auth_requests_total",
        "Total number of authorization requests"
    ).unwrap();

    /// 登录成功总数
    pub static ref AUTH_LOGIN_SUCCESS_TOTAL: Counter = Counter::new(
        "oidc_auth_login_success_total",
        "Total number of successful login attempts"
    ).unwrap();

    /// 登录失败总数
    pub static ref AUTH_LOGIN_FAILURE_TOTAL: Counter = Counter::new(
        "oidc_auth_login_failure_total",
        "Total number of failed login attempts"
    ).unwrap();

    /// 注册成功总数
    pub static ref AUTH_REGISTRATION_SUCCESS_TOTAL: Counter = Counter::new(
        "oidc_auth_registration_success_total",
        "Total number of successful registrations"
    ).unwrap();

    /// 注册失败总数
    pub static ref AUTH_REGISTRATION_FAILURE_TOTAL: Counter = Counter::new(
        "oidc_auth_registration_failure_total",
        "Total number of failed registrations"
    ).unwrap();

    /// 账户锁定总数
    pub static ref AUTH_ACCOUNT_LOCKED_TOTAL: Counter = Counter::new(
        "oidc_auth_account_locked_total",
        "Total number of account lockouts due to failed attempts"
    ).unwrap();

    // ===== Token 相关指标 =====

    /// Token 发放总数
    pub static ref TOKEN_ISSUED_TOTAL: Counter = Counter::new(
        "oidc_token_issued_total",
        "Total number of tokens issued",
    ).unwrap();

    /// Token 刷新总数
    pub static ref TOKEN_REFRESH_TOTAL: Counter = Counter::new(
        "oidc_token_refresh_total",
        "Total number of token refreshes",
    ).unwrap();

    /// Refresh Token 重放攻击检测总数
    pub static ref REPLAY_DETECTED_TOTAL: Counter = Counter::new(
        "oidc_replay_detected_total",
        "Total number of replay attacks detected",
    ).unwrap();

    /// Authorization Code 重放攻击检测总数
    pub static ref AUTH_CODE_REPLAY_TOTAL: Counter = Counter::new(
        "oidc_auth_code_replay_total",
        "Total number of authorization code replay attacks detected",
    ).unwrap();

    // ===== Session 相关指标 =====

    /// 活跃会话数
    pub static ref SESSION_ACTIVE_TOTAL: Gauge = Gauge::new(
        "session_active_total",
        "Current number of active sessions",
    ).unwrap();

    /// 会话创建总数
    pub static ref SESSION_CREATED_TOTAL: Counter = Counter::new(
        "session_created_total",
        "Total number of sessions created",
    ).unwrap();

    /// 会话销毁总数
    pub static ref SESSION_DESTROYED_TOTAL: Counter = Counter::new(
        "session_destroyed_total",
        "Total number of sessions destroyed",
    ).unwrap();

    // ===== HTTP 请求相关指标 =====

    /// HTTP 请求持续时间直方图
    pub static ref HTTP_REQUEST_DURATION_SECONDS: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
        ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
    ).unwrap();

    /// HTTP 请求总数（按路径和状态码）
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total number of HTTP requests",
    ).unwrap();

    // ===== 健康检查相关指标 =====

    /// 数据库连接池状态
    pub static ref DB_POOL_SIZE: Gauge = Gauge::new(
        "db_pool_size",
        "Current database connection pool size",
    ).unwrap();

    pub static ref DB_POOL_ACQUIRE_TIMEOUT_TOTAL: Counter = Counter::new(
        "db_pool_acquire_timeout_total",
        "Total number of database connection acquire timeouts",
    ).unwrap();

    /// JWK 密钥状态
    pub static ref JWK_KEYS_ACTIVE: Gauge = Gauge::new(
        "jwk_keys_active",
        "Number of active JWK keys",
    ).unwrap();
}

static INIT: OnceLock<()> = OnceLock::new();

/// 初始化所有 Prometheus 指标（幂等操作）
pub fn init() {
    INIT.get_or_init(|| {
        // 注册所有指标
        let _ = REGISTRY.register(Box::new(AUTH_REQUESTS_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(AUTH_LOGIN_SUCCESS_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(AUTH_LOGIN_FAILURE_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(AUTH_ACCOUNT_LOCKED_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(AUTH_REGISTRATION_SUCCESS_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(AUTH_REGISTRATION_FAILURE_TOTAL.clone()));

        let _ = REGISTRY.register(Box::new(TOKEN_ISSUED_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(TOKEN_REFRESH_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(REPLAY_DETECTED_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(AUTH_CODE_REPLAY_TOTAL.clone()));

        let _ = REGISTRY.register(Box::new(SESSION_ACTIVE_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(SESSION_CREATED_TOTAL.clone()));
        let _ = REGISTRY.register(Box::new(SESSION_DESTROYED_TOTAL.clone()));

        let _ = REGISTRY.register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone()));
        let _ = REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone()));

        let _ = REGISTRY.register(Box::new(DB_POOL_SIZE.clone()));
        let _ = REGISTRY.register(Box::new(DB_POOL_ACQUIRE_TIMEOUT_TOTAL.clone()));

        let _ = REGISTRY.register(Box::new(JWK_KEYS_ACTIVE.clone()));
    });
}

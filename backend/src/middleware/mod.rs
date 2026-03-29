// 中间件模块
//
// 提供各种中间件功能

pub mod auth;
pub mod log_redaction;
pub mod rate_limit;
pub mod request_context;

// 重新导出常用类型和函数
pub use request_context::{
    request_context_middleware, RequestContext, REQUEST_ID_HEADER, CORRELATION_ID_HEADER,
};
pub use log_redaction::{
    LogRedactionLayer, SensitiveValueRedactor,
};
pub use rate_limit::{
    RateLimiter, RateLimitConfig, rate_limit_middleware, create_rate_limiter,
};

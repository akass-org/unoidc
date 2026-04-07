// 请求上下文中间件
//
// 为每个请求添加请求 ID（request ID）和关联 ID（correlation ID）
// 用于追踪请求链路和日志关联

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{HeaderValue, Response},
    middleware::Next,
};
use std::{net::SocketAddr, time::Instant};
use tracing::field;
use uuid::Uuid;

/// 请求 ID header 名称
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// 关联 ID header 名称
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// 请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 请求 ID（唯一标识符）
    pub request_id: String,

    /// 关联 ID（用于跨服务追踪）
    pub correlation_id: Option<String>,
}

impl RequestContext {
    /// 从 HTTP 请求中提取或创建请求上下文
    pub fn from_request(req: &axum::http::Request<Body>) -> Self {
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .filter(|s| {
                s.parse::<uuid::Uuid>().is_ok() || s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let correlation_id = req
            .headers()
            .get(CORRELATION_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .filter(|s| s.len() <= 128) // 限制长度防止 HTTP 头过大
            .filter(|s| s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.~".contains(c)))
            .map(|s| s.to_string());

        Self {
            request_id,
            correlation_id,
        }
    }
}

/// 请求上下文中间件
///
/// 为每个请求：
/// 1. 生成或提取请求 ID
/// 2. 提取关联 ID（如果存在）
/// 3. 将请求 ID 添加到响应 header
/// 4. 将请求 ID 注入到 tracing span 中
pub async fn request_context_middleware(
    req: Request,
    next: Next,
) -> Response<Body> {
    let started_at = Instant::now();

    // Extract or create request context
    let ctx = RequestContext::from_request(&req);
    let method = req.method().clone();
    let uri = req.uri().clone();
    let remote_addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.to_string())
        .unwrap_or_else(|| "-".to_string());

    // 将请求 ID 添加到 tracing span
    let span = tracing::info_span!(
        "request",
        request_id = %ctx.request_id,
        correlation_id = field::Empty,
    );

    if let Some(ref corr_id) = ctx.correlation_id {
        span.record("correlation_id", corr_id.as_str());
    }

    // 在 span 中执行后续处理
    let response = span.in_scope(|| async {
        // 将请求上下文添加到请求扩展中
        let mut req = req;
        req.extensions_mut().insert(ctx.clone());

        // 执行后续中间件和处理器
        next.run(req).await
    })
    .await;

    // 将请求 ID 添加到响应 header
    let mut response = response;
    if let Ok(header_value) = HeaderValue::from_str(&ctx.request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, header_value);
    }

    // 如果有关联 ID，也添加到响应 header
    if let Some(ref corr_id) = ctx.correlation_id {
        if let Ok(header_value) = HeaderValue::from_str(corr_id) {
            response
                .headers_mut()
                .insert(CORRELATION_ID_HEADER, header_value);
        }
    }

    let status = response.status();
    let elapsed_ms = started_at.elapsed().as_millis();
    let correlation_id = ctx.correlation_id.as_deref().unwrap_or("-");

    if status.is_server_error() {
        tracing::warn!(
            target: "http.access",
            request_id = %ctx.request_id,
            correlation_id = %correlation_id,
            method = %method,
            uri = %uri,
            status = status.as_u16(),
            elapsed_ms = elapsed_ms,
            remote_addr = %remote_addr,
            "request completed"
        );
    } else {
        tracing::info!(
            target: "http.access",
            request_id = %ctx.request_id,
            correlation_id = %correlation_id,
            method = %method,
            uri = %uri,
            status = status.as_u16(),
            elapsed_ms = elapsed_ms,
            remote_addr = %remote_addr,
            "request completed"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::Request,
        Router,
        routing::get,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_request_context_generates_request_id() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(request_context_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 应该有请求 ID header
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn test_request_context_preserves_request_id() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(request_context_middleware));

        let custom_request_id = "my-custom-request-id";

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(REQUEST_ID_HEADER, custom_request_id)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 应该使用我们提供的请求 ID
        let request_id = response
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .unwrap();

        assert_eq!(request_id, custom_request_id);
    }

    #[tokio::test]
    async fn test_request_context_preserves_correlation_id() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(request_context_middleware));

        let correlation_id = "my-correlation-id";

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(CORRELATION_ID_HEADER, correlation_id)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 应该有关联 ID header
        let resp_corr_id = response
            .headers()
            .get(CORRELATION_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .unwrap();

        assert_eq!(resp_corr_id, correlation_id);
    }
}

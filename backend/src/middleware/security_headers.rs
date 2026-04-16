// Content-Security-Policy 中间件
//
// 为所有响应添加安全相关的 HTTP 头

use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;

/// 为响应添加安全相关的 HTTP 头
///
/// 包括:
/// - Content-Security-Policy: 限制资源加载来源
/// - X-Content-Type-Options: 防止 MIME 嗅探
/// - X-Frame-Options: 防止点击劫持
/// - Referrer-Policy: 控制 Referer 泄露
/// - Permissions-Policy: 限制浏览器 API 使用
pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // CSP: 允许同源资源 + 内联样式(UnoCSS 需要) + 图片 data: 和 https:
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'",
        ),
    );

    // 防止浏览器猜测 MIME 类型
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );

    // 防止嵌入 iframe（点击劫持保护）
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));

    // 控制 Referer 头泄露
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // 限制浏览器 API
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
    );

    response
}

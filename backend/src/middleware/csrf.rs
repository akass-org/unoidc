use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

const CSRF_COOKIE_NAME: &str = "unoidc_csrf";
const CSRF_HEADER_NAME: &str = "x-csrf-token";

fn is_state_changing_method(method: &Method) -> bool {
    matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH)
}

fn is_exempt_path(path: &str) -> bool {
    matches!(
        path,
        "/token"
            | "/api/v1/auth/login"
            | "/api/v1/auth/register"
            | "/api/v1/auth/logout"
            | "/api/v1/auth/forgot-password"
    )
}

pub fn extract_csrf_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    cookie_header
        .split(';')
        .find_map(|c| {
            let c = c.trim();
            c.strip_prefix(&format!("{}=", CSRF_COOKIE_NAME))
                .map(|v| v.to_string())
        })
}

pub fn extract_csrf_header(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(CSRF_HEADER_NAME)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

pub async fn csrf_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();

    if !is_state_changing_method(request.method()) || is_exempt_path(&path) {
        return next.run(request).await;
    }

    let cookie_token = extract_csrf_cookie(request.headers());
    let header_token = extract_csrf_header(request.headers());

    match (cookie_token, header_token) {
        (Some(ct), Some(ht)) if ct == ht => next.run(request).await,
        _ => {
            tracing::warn!(
                method = %request.method(),
                path = %path,
                "CSRF validation failed: token mismatch or missing"
            );
            (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "error": "CSRF validation failed",
                    "error_code": "CSRF_TOKEN_MISMATCH",
                    "status": 403,
                })),
            )
                .into_response()
        }
    }
}

pub fn generate_csrf_cookie(token: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    // 开发环境使用 Lax 允许跨端口，生产环境使用 Strict
    let same_site = if secure { "Strict" } else { "Lax" };
    // CSRF cookie 不能设置 HttpOnly，因为前端 JS 需要读取它
    format!(
        "{}={}; Path=/; SameSite={}{}",
        CSRF_COOKIE_NAME, token, same_site, secure_flag
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_state_changing_method() {
        assert!(is_state_changing_method(&Method::POST));
        assert!(is_state_changing_method(&Method::PUT));
        assert!(is_state_changing_method(&Method::DELETE));
        assert!(is_state_changing_method(&Method::PATCH));
        assert!(!is_state_changing_method(&Method::GET));
        assert!(!is_state_changing_method(&Method::OPTIONS));
        assert!(!is_state_changing_method(&Method::HEAD));
    }

    #[test]
    fn test_is_exempt_path() {
        assert!(is_exempt_path("/token"));
        assert!(is_exempt_path("/api/v1/auth/login"));
        assert!(is_exempt_path("/api/v1/auth/register"));
        assert!(is_exempt_path("/api/v1/auth/logout"));
        assert!(!is_exempt_path("/authorize/consent"));
    }

    #[test]
    fn test_extract_csrf_cookie() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("cookie", "unoidc_csrf=abc123; other=xyz".parse().unwrap());
        assert_eq!(extract_csrf_cookie(&headers), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_csrf_cookie_missing() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("cookie", "other=xyz".parse().unwrap());
        assert_eq!(extract_csrf_cookie(&headers), None);
    }

    #[test]
    fn test_extract_csrf_header() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-csrf-token", "abc123".parse().unwrap());
        assert_eq!(extract_csrf_header(&headers), Some("abc123".to_string()));
    }

    #[test]
    fn test_generate_csrf_cookie() {
        let cookie = generate_csrf_cookie("test-token", false);
        assert_eq!(cookie, "unoidc_csrf=test-token; Path=/; SameSite=Lax");

        let secure_cookie = generate_csrf_cookie("test-token", true);
        assert_eq!(secure_cookie, "unoidc_csrf=test-token; Path=/; SameSite=Strict; Secure");
    }
}

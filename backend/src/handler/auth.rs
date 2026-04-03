use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::ConnectInfo;
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    crypto,
    error::{AppError, OidcErrorCode, Result},
    metrics,
    middleware::{csrf::generate_csrf_cookie, request_context::RequestContext},
    repo::SettingsRepo,
    service::{AuditService, AuthService, UserService},
    AppState,
};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 100, message = "username must be 1-100 characters"))]
    pub username: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 100, message = "username must be 3-100 characters"))]
    pub username: String,

    #[validate(email(message = "invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, max = 128, message = "password must be 8-128 characters"))]
    pub password: String,

    #[validate(length(min = 1, max = 100, message = "display name must be 1-100 characters"))]
    pub display_name: Option<String>,

    #[validate(length(max = 100, message = "given name must not exceed 100 characters"))]
    pub given_name: Option<String>,

    #[validate(length(max = 100, message = "family name must not exceed 100 characters"))]
    pub family_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

/// 构建带签名的 session cookie
///
/// 格式: session_id.signature
fn build_cookie_value(session_id: &str, cookie_domain: Option<&String>, secure: bool, session_secret: &str) -> String {
    let signature = crypto::sign_session(session_id, session_secret).unwrap_or_default();
    let cookie_content = format!("{}.{}", session_id, signature);
    let secure_flag = if secure { "; Secure" } else { "" };
    // 开发环境使用 Lax 以允许跨端口请求，生产环境使用 Strict
    let same_site = if secure { "Strict" } else { "Lax" };
    let mut cookie = format!(
        "unoidc_session={}; HttpOnly{}; SameSite={}; Path=/",
        cookie_content, secure_flag, same_site
    );
    
    // Domain 配置说明：
    // - 只在 HTTPS 环境（secure=true）设置 Domain，localhost 设置 Domain 会导致 cookie 不工作
    // - 未配置 Domain 且 HTTPS 环境下，cookie 仅限于当前主机名，不会跨子域名
    // - 跨子域名场景需要明确配置 Domain 参数
    if secure {
        if let Some(domain) = cookie_domain {
            cookie = format!("{}; Domain={}", cookie, domain);
        } else {
            // 生产环境 (HTTPS) 未配置 Domain 时的警告
            tracing::warn!(
                "Production environment (secure=true) without Domain configured - \
                 session cookie will only work on exact hostname, not across subdomains"
            );
        }
    }
    cookie
}

/// 从 Cookie 头中提取并验证 session
///
/// 验证签名，防止 session ID 被伪造
pub fn extract_session_cookie(headers: &HeaderMap, session_secret: &str) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    let cookie_value = extract_cookie_value(cookie_header, "unoidc_session")?;

    // 分割 session_id 和签名
    let (session_id, signature) = cookie_value.split_once('.')?;

    // 验证签名
    if !crypto::verify_session_signature(session_id, signature, session_secret) {
        tracing::warn!("Invalid session cookie signature");
        return None;
    }

    Some(session_id.to_string())
}

/// 从 cookie 字符串中提取指定 cookie 的值
///
/// 先 split '=' 再比较 name，避免 `session` 匹配到 `session_id`
fn extract_cookie_value(cookie_str: &str, name: &str) -> Option<String> {
    cookie_str
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            let (cookie_name, value) = cookie.split_once('=')?;
            if cookie_name.trim() == name {
                Some(value.to_string())
            } else {
                None
            }
        })
}

/// 判断是否应该使用 Secure cookie
///
/// 基于 issuer URL 是否使用 HTTPS
fn is_secure_context(issuer: &str) -> bool {
    issuer.starts_with("https://")
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    // Try to get IP from proxy headers first, fallback to socket address
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| Some(addr.ip().to_string()));
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let result = AuthService::login(
        &state.db,
        &req.username,
        &req.password,
        ip_address.clone(),
        user_agent.clone(),
    )
    .await;

    match result {
        Ok((user, session)) => {
            let _ = AuditService::log_login_success(
                &state.db,
                user.id,
                &session.session_id,
                req_ctx.correlation_id.clone(),
                ip_address,
                user_agent,
            )
            .await;

            metrics::AUTH_LOGIN_SUCCESS_TOTAL.inc();
            metrics::SESSION_CREATED_TOTAL.inc();

            let secure = is_secure_context(&state.config.issuer);
            let session_cookie =
                build_cookie_value(&session.session_id, state.config.cookie_domain.as_ref(), secure, &state.config.session_secret);
            
            // 生成 CSRF token
            let csrf_token = crypto::generate_csrf_token()?;
            let csrf_cookie = generate_csrf_cookie(&csrf_token, secure);

            // 构建响应，手动添加多个 Set-Cookie header
            let body = serde_json::to_string(&LoginResponse {
                success: true,
                message: "Login successful".to_string(),
            }).map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("JSON_ERROR: {}", e)),
            })?;
            
            let response = Response::builder()
                .header(header::SET_COOKIE, session_cookie)
                .header(header::SET_COOKIE, csrf_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(body))
                .map_err(|e| AppError::InternalServerError {
                    error_code: Some(format!("RESPONSE_BUILD_ERROR: {}", e)),
                })?;
            
            Ok(response)
        }
        Err(e) => {
            let reason_code = match &e {
                AppError::InvalidCredentials => "invalid_credentials",
                AppError::Forbidden { .. } => "account_locked_or_disabled",
                _ => "unknown_error",
            };
            let _ = AuditService::log_login_failure(
                &state.db,
                &req.username,
                reason_code,
                req_ctx.correlation_id.clone(),
                ip_address,
                user_agent,
            )
            .await;

            metrics::AUTH_LOGIN_FAILURE_TOTAL.inc();

            Err(e)
        }
    }
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let session_id = extract_session_cookie(&headers, &state.config.session_secret)
        .ok_or(AppError::Unauthorized {
            reason: Some("No valid session cookie".to_string()),
        })?;

    // Try to get IP from proxy headers first, fallback to socket address
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| Some(addr.ip().to_string()));
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_id = crate::repo::SessionRepo::find_by_session_id(&state.db, &session_id)
        .await
        .ok()
        .and_then(|s| s)
        .map(|s| s.user_id);

    let _ = AuditService::log_logout(
        &state.db,
        user_id,
        &session_id,
        req_ctx.correlation_id.clone(),
        ip_address,
        user_agent,
    )
    .await;

    metrics::SESSION_DESTROYED_TOTAL.inc();

    AuthService::logout(&state.db, &session_id).await?;

    let secure = is_secure_context(&state.config.issuer);
    let secure_flag = if secure { "; Secure" } else { "" };
    let same_site = if secure { "Strict" } else { "Lax" };
    let mut cookie_value = format!(
        "unoidc_session=; HttpOnly{}; SameSite={}; Path=/; Max-Age=0",
        secure_flag, same_site
    );
    // 只在 HTTPS 环境设置 Domain
    if secure {
        if let Some(domain) = &state.config.cookie_domain {
            cookie_value = format!("{}; Domain={}", cookie_value, domain);
        }
    }

    Ok((
        [(header::SET_COOKIE, cookie_value)],
        Json(LogoutResponse {
            success: true,
            message: "Logout successful".to_string(),
        }),
    )
        .into_response())
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Response> {
    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    // Try to get IP from proxy headers first, fallback to socket address
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| Some(addr.ip().to_string()));
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let username = req.username.clone();

    match UserService::register(
        &state.db,
        req.username,
        req.email,
        req.password,
        req.display_name,
    )
    .await
    {
        Ok(user) => {
            let _ = AuditService::log_user_created(
                &state.db,
                user.id,
                &user.username,
                req_ctx.correlation_id.clone(),
                ip_address,
                user_agent,
            )
            .await;

            metrics::AUTH_REGISTRATION_SUCCESS_TOTAL.inc();

            Ok((
                Json(LoginResponse {
                    success: true,
                    message: "Registration successful".to_string(),
                }),
            )
                .into_response())
        }
        Err(e) => {
            let _ = AuditService::log_registration_failure(
                &state.db,
                &username,
                &e.to_string(),
                req_ctx.correlation_id.clone(),
                ip_address,
                user_agent,
            )
            .await;

            metrics::AUTH_REGISTRATION_FAILURE_TOTAL.inc();

            Err(AppError::BusinessError {
                code: "REGISTRATION_FAILED".to_string(),
                message: e.to_string(),
            })
        }
    }
}

pub async fn forgot_password() -> Result<Json<LoginResponse>> {
    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Password reset not yet implemented".to_string()),
    })
}

/// 获取当前会话信息
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub picture: Option<String>,
    pub is_admin: bool,
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>> {
    let session_id = extract_session_cookie(&headers, &state.config.session_secret)
        .ok_or(AppError::Unauthorized {
            reason: Some("No valid session".to_string()),
        })?;

    let session = crate::repo::SessionRepo::find_by_session_id(&state.db, &session_id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?
        .ok_or(AppError::Unauthorized {
            reason: Some("Session not found".to_string()),
        })?;

    // Check if session is expired
    if session.is_expired() {
        return Err(AppError::Unauthorized {
            reason: Some("Session expired".to_string()),
        });
    }

    let user = crate::repo::UserRepo::find_by_id(&state.db, session.user_id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?
        .ok_or(AppError::Unauthorized {
            reason: Some("User not found".to_string()),
        })?;

    // 检查用户是否在 admin 组
    let is_admin = check_user_admin(&state.db, user.id).await.unwrap_or(false);

    Ok(Json(SessionResponse {
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            display_name: user.display_name.unwrap_or_default(),
            picture: user.picture,
            is_admin,
        },
    }))
}

/// 检查用户是否为管理员
async fn check_user_admin(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> anyhow::Result<bool> {
    // 获取 admin 组
    let admin_group = match crate::repo::GroupRepo::find_by_name(pool, "admin").await? {
        Some(g) => g,
        None => return Ok(false),
    };

    // 检查用户是否在 admin 组
    let user_groups = crate::repo::GroupRepo::find_user_groups(pool, user_id).await?;
    Ok(user_groups.iter().any(|g| g.id == admin_group.id))
}

/// 公共配置响应（用于登录页）
#[derive(Debug, Serialize)]
pub struct PublicConfigResponse {
    pub brand_name: String,
    pub logo_url: String,
    pub login_background_url: String,
    pub login_layout: String,
}

/// 获取公共配置（无需登录，用于登录页）
pub async fn get_public_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PublicConfigResponse>> {
    // 从数据库读取设置
    let settings = SettingsRepo::get_all(&state.db).await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DB_ERROR: {}", e)),
        })?;
    
    // 转换为 map 方便查找
    let settings_map: std::collections::HashMap<String, String> = 
        settings.into_iter().collect();
    
    let get_value = |key: &str, default: &str| -> String {
        settings_map.get(key).cloned().unwrap_or_else(|| default.to_string())
    };
    
    Ok(Json(PublicConfigResponse {
        brand_name: get_value("brand_name", "UNOIDC"),
        logo_url: get_value("logo_url", ""),
        login_background_url: get_value("login_background_url", ""),
        login_layout: get_value("login_layout", "split-left"),
    }))
}

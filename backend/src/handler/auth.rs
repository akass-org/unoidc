// 认证 HTTP 处理器
//
// 处理登录、登出等认证相关的 HTTP 请求

use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use validator::Validate;

use crate::{
    error::{AppError, OidcErrorCode, Result},
    metrics,
    middleware::request_context::RequestContext,
    service::{AuditService, AuthService},
    AppState,
};

/// 登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 100, message = "username must be 1-100 characters"))]
    pub username: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
}

/// 注册请求
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 100, message = "username must be 3-100 characters"))]
    pub username: String,

    #[validate(email(message = "invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, max = 128, message = "password must be 8-128 characters"))]
    pub password: String,

    #[validate(length(max = 100, message = "given name must not exceed 100 characters"))]
    pub given_name: Option<String>,

    #[validate(length(max = 100, message = "family name must not exceed 100 characters"))]
    pub family_name: Option<String>,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
}

/// 登出响应
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

/// 用户登录
///
/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Extension(addr): Extension<Option<SocketAddr>>,
    Extension(req_ctx): Extension<RequestContext>,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    // Input validation
    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    // Get client info
    let ip_address = addr.map(|a| a.to_string());
    let user_agent = None; // TODO: Extract from request header

    // Call auth service
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
            // Audit log
            let _ = AuditService::log_login_success(
                &state.db,
                user.id,
                &session.session_id,
                req_ctx.correlation_id.clone(),
                ip_address,
                user_agent,
            ).await;

            // Metrics
            metrics::AUTH_LOGIN_SUCCESS_TOTAL.inc();
            metrics::SESSION_CREATED_TOTAL.inc();

            // Cookie
            let cookie_value = format!(
                "unoidc_session={}; HttpOnly; Secure; SameSite=Strict; Path=/",
                session.session_id
            );

            Ok((
                [(header::SET_COOKIE, cookie_value)],
                Json(LoginResponse {
                    success: true,
                    message: "Login successful".to_string(),
                }),
            )
                .into_response())
        }
        Err(e) => {
            // Audit log
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
            ).await;

            // Metrics
            metrics::AUTH_LOGIN_FAILURE_TOTAL.inc();

            Err(e)
        }
    }
}

/// 用户登出
///
/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(addr): Extension<Option<SocketAddr>>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let session_id = crate::middleware::auth::extract_session_cookie(&headers)
        .ok_or(AppError::Unauthorized {
            reason: Some("No session cookie".to_string()),
        })?;

    let ip_address = addr.map(|a| a.to_string());
    let user_agent = None;

    // 查找 session 获取 user_id
    let user_id = crate::repo::SessionRepo::find_by_session_id(&state.db, &session_id)
        .await
        .ok()
        .and_then(|s| s)
        .map(|s| s.user_id);

    // 记录登出审计日志
    let _ = AuditService::log_logout(
        &state.db,
        user_id,
        &session_id,
        req_ctx.correlation_id.clone(),
        ip_address,
        user_agent,
    ).await;

    // 更新指标
    metrics::SESSION_DESTROYED_TOTAL.inc();

    AuthService::logout(&state.db, &session_id).await?;

    // 清除 Cookie（设置过期时间为过去）
    let cookie_value = "unoidc_session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0";

    Ok((
        [(header::SET_COOKIE, cookie_value)],
        Json(LogoutResponse {
            success: true,
            message: "Logout successful".to_string(),
        }),
    )
        .into_response())
}

/// 用户注册
///
/// POST /api/v1/auth/register
pub async fn register(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>> {
    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Registration not yet implemented".to_string()),
    })
}

/// 忘记密码
///
/// POST /api/v1/auth/forgot-password
pub async fn forgot_password() -> Result<Json<LoginResponse>> {
    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Password reset not yet implemented".to_string()),
    })
}

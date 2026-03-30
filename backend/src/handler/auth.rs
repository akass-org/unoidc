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
    service::AuthService,
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
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    // 输入验证
    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    // 获取客户端信息
    let ip_address = addr.map(|a| a.to_string());
    let user_agent = None; // TODO: 从请求头中提取 User-Agent

    // 调用认证服务进行登录
    let (_user, session) = AuthService::login(
        &state.db,
        &req.username,
        &req.password,
        ip_address,
        user_agent,
    )
    .await
    .map_err(|e| {
        // 登录失败，返回相应的错误
        match e {
            AppError::InvalidCredentials => AppError::InvalidCredentials,
            AppError::Forbidden { reason } => AppError::Forbidden { reason },
            _ => AppError::InternalServerError { error_code: None },
        }
    })?;

    // 构造安全的 Cookie
    let cookie_value = format!(
        "unoidc_session={}; HttpOnly; Secure; SameSite=Strict; Path=/",
        session.session_id
    );

    // 返回响应（带 Set-Cookie 头）
    Ok((
        [(header::SET_COOKIE, cookie_value)],
        Json(LoginResponse {
            success: true,
            message: "Login successful".to_string(),
        }),
    )
        .into_response())
}

/// 用户登出
///
/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response> {
    let session_id = crate::middleware::auth::extract_session_cookie(&headers)
        .ok_or(AppError::Unauthorized {
            reason: Some("No session cookie".to_string()),
        })?;

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

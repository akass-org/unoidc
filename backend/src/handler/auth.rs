// 认证 HTTP 处理器
//
// 处理登录、登出等认证相关的 HTTP 请求

use axum::{
    extract::State,
    http::HeaderMap,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

use crate::{
    error::{AppError, OidcErrorCode, Result},
    service::AuthService,
    AppState,
};

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
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
) -> Result<Json<LoginResponse>> {
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

    // 登录成功，返回会话ID
    Ok(Json(LoginResponse {
        success: true,
        message: "Login successful".to_string(),
        session_id: Some(session.session_id),
    }))
}

/// 用户登出
///
/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<LogoutResponse>> {
    let session_id = crate::middleware::auth::extract_session_cookie(&headers)
        .ok_or(AppError::Unauthorized {
            reason: Some("No session cookie".to_string()),
        })?;

    AuthService::logout(&state.db, &session_id).await?;

    Ok(Json(LogoutResponse {
        success: true,
        message: "Logout successful".to_string(),
    }))
}

/// 用户注册
///
/// POST /api/v1/auth/register
pub async fn register() -> Result<Json<LoginResponse>> {
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

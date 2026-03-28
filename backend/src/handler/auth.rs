// 认证 HTTP 处理器
//
// 处理登录、登出等认证相关的 HTTP 请求

use axum::{
    extract::State,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

use crate::{
    error::{AppError, Result},
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
    Extension(session_id): Extension<String>,
) -> Result<Json<LogoutResponse>> {
    // 调用认证服务登出
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
    // TODO: 实现注册逻辑
    Ok(Json(LoginResponse {
        success: true,
        message: "Registration successful".to_string(),
        session_id: None,
    }))
}

/// 忘记密码
///
/// POST /api/v1/auth/forgot-password
pub async fn forgot_password() -> Result<Json<LoginResponse>> {
    // TODO: 实现忘记密码逻辑
    Ok(Json(LoginResponse {
        success: true,
        message: "Password reset email sent".to_string(),
        session_id: None,
    }))
}

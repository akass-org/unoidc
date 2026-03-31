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

fn build_cookie_value(session_id: &str, cookie_domain: Option<&String>, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    let mut cookie = format!(
        "unoidc_session={}; HttpOnly{}; SameSite=Strict; Path=/",
        session_id, secure_flag
    );
    if let Some(domain) = cookie_domain {
        cookie = format!("{}; Domain={}", cookie, domain);
    }
    cookie
}

/// 判断是否应该使用 Secure cookie
///
/// 基于 issuer URL 是否使用 HTTPS
fn is_secure_context(issuer: &str) -> bool {
    issuer.starts_with("https://")
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Extension(addr): Extension<Option<SocketAddr>>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    req.validate().map_err(|e| AppError::ValidationError {
        field: "request".to_string(),
        message: e.to_string(),
    })?;

    let ip_address = addr.map(|a| a.to_string());
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
            let cookie_value =
                build_cookie_value(&session.session_id, state.config.cookie_domain.as_ref(), secure);

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
    Extension(addr): Extension<Option<SocketAddr>>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
) -> Result<Response> {
    let session_id = crate::middleware::auth::extract_session_cookie(&headers)
        .ok_or(AppError::Unauthorized {
            reason: Some("No session cookie".to_string()),
        })?;

    let ip_address = addr.map(|a| a.to_string());
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
    let mut cookie_value = format!(
        "unoidc_session=; HttpOnly{}; SameSite=Strict; Path=/; Max-Age=0",
        secure_flag
    );
    if let Some(domain) = &state.config.cookie_domain {
        cookie_value = format!("{}; Domain={}", cookie_value, domain);
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
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>> {
    // M-21: 跳过验证，因为注册功能尚未实现
    // 功能实现后应添加验证：req.validate()...

    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Registration not yet implemented".to_string()),
    })
}

pub async fn forgot_password() -> Result<Json<LoginResponse>> {
    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Password reset not yet implemented".to_string()),
    })
}

// Passkey Handler
//
// Passkey 管理 API 接口

use axum::{
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use webauthn_rs::prelude::*;
use uuid::Uuid;
use webauthn_rs_proto::{
    AuthenticationExtensionsClientOutputs, AuthenticatorAssertionResponseRaw,
    AuthenticatorAttestationResponseRaw, RegistrationExtensionsClientOutputs,
};

use crate::{
    error::{AppError, Result},
    handler::auth::{build_login_session_response, extract_client_ip_secure},
    middleware::{auth::require_auth_user, request_context::RequestContext},
    repo::SettingsRepo,
    service::PasskeyService,
    AppState,
};
use tracing::{info, warn};

/// 列出当前用户的所有 passkey
pub async fn list_passkeys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;
    let credentials = PasskeyService::list_credentials(&state, auth_user.user.id).await?;
    Ok(Json(credentials))
}

/// 开始注册 passkey
pub async fn start_register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;
    info!(user_id = %auth_user.user.id, "Starting passkey registration");
    let display_name = auth_user
        .user
        .display_name
        .as_ref()
        .unwrap_or(&auth_user.user.username)
        .clone();
    let ccr = PasskeyService::start_registration(
        &state,
        auth_user.user.id,
        &auth_user.user.username,
        &display_name,
    )
    .await?;
    info!(user_id = %auth_user.user.id, "Passkey registration started successfully");
    Ok(Json(ccr))
}

/// 完成注册 passkey 的请求体
#[derive(Debug, Deserialize)]
pub struct FinishRegisterRequest {
    pub id: String,
    #[serde(rename = "rawId")]
    pub raw_id: Base64UrlSafeData,
    pub response: AuthenticatorAttestationResponseRaw,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, alias = "clientExtensionResults", alias = "extensions")]
    pub client_extension_results: RegistrationExtensionsClientOutputs,
}

/// 完成注册 passkey
pub async fn finish_register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<FinishRegisterRequest>,
) -> Result<impl IntoResponse> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;

    let reg = RegisterPublicKeyCredential {
        id: req.id,
        raw_id: req.raw_id,
        response: req.response,
        extensions: req.client_extension_results,
        type_: req.type_,
    };

    match PasskeyService::finish_registration(&state, auth_user.user.id, &reg).await {
        Ok(()) => {
            info!(user_id = %auth_user.user.id, "Passkey registration finished successfully");
            Ok(Json(serde_json::json!({ "success": true })))
        }
        Err(e) => {
            warn!(user_id = %auth_user.user.id, error = %e, "Passkey registration failed");
            Err(e)
        }
    }
}

/// 完成 passkey 登录的请求体
#[derive(Debug, Deserialize)]
pub struct FinishLoginRequest {
    pub id: String,
    #[serde(rename = "rawId")]
    pub raw_id: Base64UrlSafeData,
    pub response: AuthenticatorAssertionResponseRaw,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, alias = "clientExtensionResults", alias = "extensions")]
    pub client_extension_results: AuthenticationExtensionsClientOutputs,
}

/// 开始 passkey 登录
pub async fn start_login(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    info!("Starting passkey authentication");
    let rcr = PasskeyService::start_authentication(&state).await?;
    info!("Passkey authentication started successfully");
    Ok(Json(rcr))
}

/// 完成 passkey 登录
pub async fn finish_login(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(req_ctx): Extension<RequestContext>,
    headers: HeaderMap,
    Json(req): Json<FinishLoginRequest>,
) -> Result<impl IntoResponse> {
    let auth = PublicKeyCredential {
        id: req.id,
        raw_id: req.raw_id,
        response: req.response,
        extensions: req.client_extension_results,
        type_: req.type_,
    };

    let ip_address = extract_client_ip_secure(&headers, &addr, &state.config.trusted_proxy_ips);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match PasskeyService::finish_authentication(&state, &auth).await {
        Ok((user, _counter)) => {
            info!(user_id = %user.id, username = %user.username, "Passkey login succeeded");
            let session_input = crate::model::CreateSession::new(
                user.id,
                ip_address.clone(),
                user_agent.clone(),
            );
            let session = crate::repo::SessionRepo::create(&state.db, session_input)
                .await
                .map_err(|e| AppError::InternalServerError {
                    error_code: Some(format!("SESSION_CREATE_ERROR: {}", e)),
                })?;

            let response = build_login_session_response(
                &state,
                &user,
                &session,
                &headers,
                &req_ctx,
                ip_address,
                user_agent,
            )
            .await?;

            Ok(response)
        }
        Err(e) => {
            warn!(error = %e, credential_id = %auth.id, "Passkey login failed");
            let reason_code = match &e {
                AppError::AuthenticationFailed { .. } => "passkey_auth_failed",
                AppError::InvalidRequest(_) => "passkey_invalid_request",
                _ => "passkey_unknown_error",
            };
            let credential_id = auth.id.clone();
            let _ = crate::service::AuditService::log_login_failure(
                &state.db,
                &credential_id,
                reason_code,
                req_ctx.correlation_id.clone(),
                ip_address,
                user_agent,
            )
            .await;
            Err(e)
        }
    }
}

/// 匿名注册开始请求
#[derive(Debug, Deserialize)]
pub struct AnonRegisterStartRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
}

/// 匿名注册开始响应
#[derive(Debug, Serialize)]
pub struct AnonRegisterStartResponse {
    pub options: CreationChallengeResponse,
    pub temp_user_id: String,
}

/// 开始匿名 passkey 注册
pub async fn start_register_anon(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnonRegisterStartRequest>,
) -> Result<impl IntoResponse> {
    let enable_passkey_signup = SettingsRepo::get(&state.db, "enable_passkey_signup")
        .await
        .ok()
        .flatten()
        .map(|v| v.parse().unwrap_or(true))
        .unwrap_or(true);
    if !enable_passkey_signup {
        return Err(AppError::Forbidden {
            reason: Some("纯 passkey 注册已被管理员禁用".to_string()),
        });
    }

    let temp_user_id = Uuid::new_v4();
    let display_name = if req.display_name.is_empty() {
        req.username.clone()
    } else {
        req.display_name
    };
    info!(username = %req.username, "Starting anonymous passkey registration");
    let ccr = PasskeyService::start_anon_registration(
        &state,
        temp_user_id,
        &req.username,
        &display_name,
    )
    .await?;
    info!(username = %req.username, "Anonymous passkey registration started successfully");
    Ok(Json(AnonRegisterStartResponse {
        options: ccr,
        temp_user_id: temp_user_id.to_string(),
    }))
}

/// 匿名注册完成请求
#[derive(Debug, Deserialize)]
pub struct AnonRegisterFinishRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub id: String,
    #[serde(rename = "rawId")]
    pub raw_id: Base64UrlSafeData,
    pub response: AuthenticatorAttestationResponseRaw,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, alias = "clientExtensionResults", alias = "extensions")]
    pub client_extension_results: RegistrationExtensionsClientOutputs,
}

/// 完成匿名 passkey 注册
pub async fn finish_register_anon(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnonRegisterFinishRequest>,
) -> Result<impl IntoResponse> {
    let enable_passkey_signup = SettingsRepo::get(&state.db, "enable_passkey_signup")
        .await
        .ok()
        .flatten()
        .map(|v| v.parse().unwrap_or(true))
        .unwrap_or(true);
    if !enable_passkey_signup {
        return Err(AppError::Forbidden {
            reason: Some("纯 passkey 注册已被管理员禁用".to_string()),
        });
    }

    let reg = RegisterPublicKeyCredential {
        id: req.id,
        raw_id: req.raw_id,
        response: req.response,
        extensions: req.client_extension_results,
        type_: req.type_,
    };

    match PasskeyService::finish_anon_registration(
        &state,
        &reg,
        req.username.clone(),
        req.email.clone(),
        req.display_name.clone(),
    )
    .await
    {
        Ok(()) => {
            info!(username = %req.username, "Anonymous passkey registration finished successfully");
            Ok(Json(serde_json::json!({ "success": true })))
        }
        Err(e) => {
            warn!(username = %req.username, error = %e, "Anonymous passkey registration failed");
            Err(e)
        }
    }
}

/// 删除 passkey
pub async fn delete_passkey(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;
    info!(user_id = %auth_user.user.id, credential_id = %id, "Deleting passkey");
    PasskeyService::delete_credential(&state, &id, auth_user.user.id).await?;
    info!(user_id = %auth_user.user.id, credential_id = %id, "Passkey deleted successfully");
    Ok(Json(serde_json::json!({ "success": true })))
}

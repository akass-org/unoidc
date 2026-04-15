// Passkey Handler
//
// Passkey 管理 API 接口

use axum::{
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use webauthn_rs::prelude::*;
use webauthn_rs_proto::{
    AuthenticationExtensionsClientOutputs, AuthenticatorAssertionResponseRaw,
    AuthenticatorAttestationResponseRaw, RegistrationExtensionsClientOutputs,
};

use crate::{
    error::{AppError, Result},
    handler::auth::{build_login_session_response, extract_client_ip_secure},
    middleware::{auth::require_auth_user, request_context::RequestContext},
    service::PasskeyService,
    AppState,
};

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

    PasskeyService::finish_registration(&state, auth_user.user.id, &reg).await?;
    Ok(Json(serde_json::json!({ "success": true })))
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
    let rcr = PasskeyService::start_authentication(&state).await?;
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

/// 删除 passkey
pub async fn delete_passkey(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let auth_user = require_auth_user(&state.db, &headers, &state.config.session_secret).await?;
    PasskeyService::delete_credential(&state, &id, auth_user.user.id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

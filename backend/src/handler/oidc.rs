// OIDC HTTP 处理器
//
// 处理 Discovery, JWKS, Authorize, Token, UserInfo 等 OIDC 端点

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::Result;
use crate::service::KeyService;
use crate::AppState;

// ============================================================
// Discovery
// ============================================================

/// GET /.well-known/openid-configuration
pub async fn discovery(State(state): State<Arc<AppState>>) -> Result<Json<Value>> {
    let base = &state.config.app_base_url;
    Ok(Json(json!({
        "issuer": state.config.issuer,
        "authorization_endpoint": format!("{}/authorize", base),
        "token_endpoint": format!("{}/token", base),
        "userinfo_endpoint": format!("{}/userinfo", base),
        "jwks_uri": format!("{}/jwks.json", base),
        "end_session_endpoint": format!("{}/logout", base),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "client_secret_basic", "none"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "profile", "email", "groups", "offline_access"],
        "id_token_signing_alg_values_supported": ["ES256"],
        "subject_types_supported": ["public"],
        "claims_supported": ["sub", "iss", "aud", "exp", "iat", "auth_time", "nonce", "acr", "amr", "name", "given_name", "family_name", "preferred_username", "email", "email_verified", "groups"],
    })))
}

// ============================================================
// JWKS
// ============================================================

/// GET /jwks.json
pub async fn jwks(State(state): State<Arc<AppState>>) -> Result<Json<Value>> {
    let keys = KeyService::get_jwks(&state.db).await
        .map_err(|e| crate::error::AppError::InternalServerError {
            error_code: Some(format!("JWKS_ERROR: {}", e)),
        })?;

    let jwk_list: Vec<Value> = keys.iter().map(|k| k.public_key_jwk.clone()).collect();
    Ok(Json(json!({ "keys": jwk_list })))
}

// ============================================================
// Authorize
// ============================================================

/// GET /authorize 查询参数
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub prompt: Option<String>,
}

/// GET /authorize — TODO: Task 10 完整实现
pub async fn authorize_get(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
    Query(_req): Query<AuthorizeRequest>,
) -> Result<&'static str> {
    Ok("authorize")
}

/// POST /authorize/consent — TODO: Task 10 完整实现
pub async fn authorize_consent() -> Result<Json<Value>> {
    Ok(Json(json!({})))
}

// ============================================================
// Token
// ============================================================

/// POST /token — TODO: Task 11 完整实现
pub async fn token() -> Result<Json<Value>> {
    Ok(Json(json!({})))
}

// ============================================================
// UserInfo
// ============================================================

/// GET /userinfo — TODO: Task 12 完整实现
pub async fn userinfo() -> Result<Json<Value>> {
    Ok(Json(json!({})))
}

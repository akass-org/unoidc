// OIDC HTTP 处理器
//
// 处理 Discovery, JWKS, Authorize, Token, UserInfo 等 OIDC 端点

use axum::{
    extract::{Query, State},
    http,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::{AppError, OidcErrorCode, Result};
use crate::metrics;
use crate::service::{KeyService, LogoutService};
use crate::crypto::jwt::{self, AccessTokenClaims};
use crate::repo::{UserRepo, GroupRepo};
use crate::AppState;
use crate::model::Jwk;

type HeaderMap = http::HeaderMap;

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
    let keys = KeyService::get_jwks(&state.db)
        .await
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

/// GET /authorize — Authorization endpoint
pub async fn authorize_get(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
    Query(_req): Query<AuthorizeRequest>,
) -> Result<&'static str> {
    metrics::AUTH_REQUESTS_TOTAL.inc();
    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Authorization endpoint not yet implemented".to_string()),
    })
}

/// POST /authorize/consent — 尚未实现
pub async fn authorize_consent() -> Result<Json<Value>> {
    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Consent endpoint not yet implemented".to_string()),
    })
}

// ============================================================
// Token
// ============================================================

/// POST /token — 尚未实现
pub async fn token() -> Result<Json<Value>> {
    Err(AppError::OidcError {
        error: OidcErrorCode::TemporarilyUnavailable,
        error_description: Some("Token endpoint not yet implemented".to_string()),
    })
}

// ============================================================
// UserInfo
// ============================================================

/// GET /userinfo
///
/// 根据 access token 返回用户信息，按 scope 过滤字段
pub async fn userinfo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    // 提取 Bearer token
    let token = extract_bearer_token(&headers)?;

    // 获取公钥用于验证 token
    let jwks = KeyService::get_jwks(&state.db)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("JWKS_ERROR: {}", e)),
        })?;

    if jwks.is_empty() {
        return Err(AppError::InternalServerError {
            error_code: Some("No signing keys available".to_string()),
        });
    }

    // 尝试用每个公钥验证 token（直到找到匹配的 kid）
    let claims = verify_access_token(&token, &jwks, &state.config.issuer)?;

    // 加载用户
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::InvalidRequest("Invalid user ID in token".to_string()))?;

    let user = UserRepo::find_by_id(&state.db, user_id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DATABASE_ERROR: {}", e)),
        })?
        .ok_or_else(|| AppError::Unauthorized {
            reason: Some("User no longer exists".to_string()),
        })?;

    // 加载用户组
    let groups: Vec<String> = GroupRepo::find_user_groups(&state.db, user.id)
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("DATABASE_ERROR: {}", e)),
        })?
        .into_iter()
        .map(|g| g.name)
        .collect();

    // 根据 scope 构建响应
    let scopes: Vec<&str> = claims.scope.split_whitespace().collect();
    let mut response = json!({
        "sub": user.id.to_string(),
    });

    // profile scope
    if scopes.contains(&"profile") {
        response["name"] = Value::String(user.get_display_name().to_string());
        if let Some(ref given_name) = user.given_name {
            response["given_name"] = Value::String(given_name.clone());
        }
        if let Some(ref family_name) = user.family_name {
            response["family_name"] = Value::String(family_name.clone());
        }
        response["preferred_username"] = Value::String(user.username.clone());
        if let Some(ref display_name) = user.display_name {
            response["display_name"] = Value::String(display_name.clone());
        }
        if let Some(ref picture) = user.picture {
            response["picture"] = Value::String(picture.clone());
        }
    }

    // email scope
    if scopes.contains(&"email") {
        response["email"] = Value::String(user.email.clone());
        response["email_verified"] = Value::Bool(user.email_verified);
    }

    // groups scope
    if scopes.contains(&"groups") {
        response["groups"] = serde_json::to_value(&groups).unwrap_or(Value::Null);
    }

    Ok(Json(response))
}

/// 从 Authorization header 提取 Bearer token
fn extract_bearer_token(headers: &HeaderMap) -> Result<String> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized {
            reason: Some("Missing Authorization header".to_string()),
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized {
            reason: Some("Authorization header must use Bearer scheme".to_string()),
        });
    }

    Ok(auth_header[7..].to_string())
}

/// 验证 access token 并返回 claims
fn verify_access_token(
    token: &str,
    jwks: &[Jwk],
    expected_issuer: &str,
) -> Result<AccessTokenClaims> {
    // 尝试用每个公钥验证（jsonwebtoken 会自动匹配 kid）
    for jwk in jwks {
        // 从 JWK JSON 转换为 PEM 格式的公钥
        let public_key_pem = match KeyService::jwk_to_public_key_pem(&jwk.public_key_jwk) {
            Ok(pem) => pem,
            Err(_) => continue, // 跳过无效的 JWK
        };

        if let Ok(token_data) = jwt::verify_jwt::<AccessTokenClaims>(
            token,
            &public_key_pem,
            Some(expected_issuer),
            None, // audience 由客户端控制，不在此验证
        ) {
            // 检查 token 类型
            if token_data.claims.token_type != "oauth-access-token" {
                continue;
            }
            return Ok(token_data.claims);
        }
    }

    Err(AppError::InvalidToken {
        reason: Some("Invalid or expired access token".to_string()),
    })
}

// ============================================================
// Logout
// ============================================================

/// GET /logout — RP-Initiated Logout
///
/// 根据 OIDC Session Management 规范处理 RP 发起登出
/// 支持 id_token_hint 和 post_logout_redirect_uri 参数
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// ID Token Hint - 包含当前用户信息的已签名 JWT
    pub id_token_hint: Option<String>,
    /// 登出后重定向 URI
    pub post_logout_redirect_uri: Option<String>,
    /// 状态参数，原样返回给客户端
    pub state: Option<String>,
}

/// GET /logout
///
/// 处理 RP-Initiated Logout 请求
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Query(req): Query<LogoutRequest>,
) -> Result<impl IntoResponse> {
    use axum::http::StatusCode;

    if let Some(ref hint) = req.id_token_hint {
        if !hint.is_empty() {
            let hint_result = LogoutService::validate_id_token_hint::<serde_json::Value>(&state.db, hint).await;

            if hint_result.is_err() {
                return Err(AppError::InvalidToken {
                    reason: Some("Invalid id_token_hint".to_string()),
                });
            }
        }
    }

    let base = &state.config.app_base_url;
    let location = if let Some(ref redirect) = req.post_logout_redirect_uri {
        if redirect.is_empty() {
            return Err(AppError::InvalidRequest(
                "post_logout_redirect_uri cannot be empty".to_string(),
            ));
        }

        let mut validated = false;

        if let Some(ref hint) = req.id_token_hint {
            if !hint.is_empty() {
                if let Ok(claims) = LogoutService::validate_id_token_hint::<serde_json::Value>(&state.db, hint).await {
                    if let Some(aud) = claims.get("aud").and_then(|v| v.as_str()) {
                        if let Ok(client_id) = uuid::Uuid::parse_str(aud) {
                            validated = LogoutService::validate_post_logout_redirect(&state.db, &client_id, redirect).await.is_ok();
                        }
                    }
                }
            }
        }

        if !validated {
            return Err(AppError::InvalidRequest(
                "post_logout_redirect_uri must be validated via id_token_hint with a registered client".to_string(),
            ));
        }

        redirect.clone()
    } else {
        base.clone()
    };

    let final_location = if let Some(ref s) = req.state {
        if location.contains('?') {
            format!("{}&state={}", location, s)
        } else {
            format!("{}?state={}", location, s)
        }
    } else {
        location
    };

    Ok::<_, AppError>((
        StatusCode::FOUND,
        [(axum::http::header::LOCATION, final_location)],
    ).into_response())
}

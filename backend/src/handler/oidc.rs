// OIDC HTTP 处理器
//
// 处理 Discovery, JWKS, Authorize, Token, UserInfo 等 OIDC 端点

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

use crate::crypto::jwt;
use crate::crypto::jwt::AccessTokenClaims;
use crate::error::{AppError, OidcErrorCode, Result};
use crate::metrics;
use crate::model::Jwk;
use crate::repo::{GroupRepo, UserRepo};
use crate::service::{AuditService, AuthService, KeyService, LogoutService};
use crate::AppState;
use std::collections::HashMap;

pub async fn discovery(State(state): State<Arc<AppState>>) -> Json<Value> {
    let issuer = state.config.issuer.clone();
    let base_url = state.config.app_base_url.clone();
    Json(json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{}/authorize", base_url),
        "token_endpoint": format!("{}/token", base_url),
        "userinfo_endpoint": format!("{}/userinfo", base_url),
        "jwks_uri": format!("{}/jwks.json", base_url),
        "end_session_endpoint": format!("{}/logout", base_url),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["ES256"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "profile", "email", "groups", "offline_access"],
        "claims_supported": ["sub", "aud", "exp", "iat", "jti", "auth_time", "amr", "nonce", "name", "given_name", "family_name", "preferred_username", "display_name", "picture", "email", "email_verified", "groups"],
    }))
}

// ============================================================
// JWKS
// ============================================================

/// GET /jwks.json
pub async fn jwks(State(state): State<Arc<AppState>>) -> Result<Json<Value>> {
    let keys = KeyService::get_jwks(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get JWKS: {}", e);
            crate::error::AppError::InternalServerError {
                error_code: Some("KEYS_UNAVAILABLE".to_string()),
            }
        })?;

    let jwk_list: Vec<Value> = keys.iter().map(|k| k.public_key_jwk.clone()).collect();
    Ok(Json(json!({ "keys": jwk_list })))
}

// ============================================================
// Authorize
// ============================================================

/// GET /authorize 查询参数
#[derive(Debug, Deserialize, Validate)]
pub struct AuthorizeRequest {
    #[validate(length(max = 50, message = "response_type too long"))]
    pub response_type: String,
    #[validate(length(max = 255, message = "client_id too long"))]
    pub client_id: String,
    #[validate(length(max = 2048, message = "redirect_uri too long"))]
    pub redirect_uri: String,
    #[validate(length(max = 500, message = "scope too long"))]
    pub scope: String,
    #[validate(length(max = 1024, message = "state too long"))]
    pub state: String,
    #[validate(length(max = 500, message = "nonce too long"))]
    pub nonce: Option<String>,
    #[validate(length(max = 500, message = "code_challenge too long"))]
    pub code_challenge: String,
    #[validate(length(max = 10, message = "code_challenge_method too long"))]
    pub code_challenge_method: String,
}

impl AuthorizeRequest {
    /// 验证请求参数
    pub fn validate_request(&self) -> Result<()> {
        use validator::Validate;

        // 基础长度验证
        self.validate().map_err(|e| AppError::ValidationError {
            field: "request".to_string(),
            message: e.to_string(),
        })?;

        // 验证 response_type 必须是 "code"
        if self.response_type != "code" {
            return Err(AppError::OidcError {
                error: OidcErrorCode::UnsupportedResponseType,
                error_description: Some(format!(
                    "response_type '{}' is not supported, only 'code' is supported",
                    self.response_type
                )),
            });
        }

        // 验证 code_challenge_method 必须是 S256
        if self.code_challenge_method != "S256" {
            return Err(AppError::OidcError {
                error: OidcErrorCode::InvalidRequest,
                error_description: Some(format!(
                    "code_challenge_method '{}' is not supported, only 'S256' is supported",
                    self.code_challenge_method
                )),
            });
        }

        Ok(())
    }
}

/// GET /authorize — Authorization endpoint
pub async fn authorize_get(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
    Query(req): Query<AuthorizeRequest>,
) -> Result<&'static str> {
    // 验证请求参数（包括长度限制和合规性检查）
    req.validate_request()?;

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
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("JWKS_UNAVAILABLE".to_string()),
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
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("USER_LOOKUP_FAILED".to_string()),
        })?
        .ok_or_else(|| AppError::Unauthorized {
            reason: Some("User no longer exists".to_string()),
        })?;

    // 加载用户组
    let groups: Vec<String> = GroupRepo::find_user_groups(&state.db, user.id)
        .await
        .map_err(|_| AppError::InternalServerError {
            error_code: Some("GROUP_LOOKUP_FAILED".to_string()),
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

    Ok(auth_header[7..].trim().to_string())
}

/// 验证 access token 并返回 claims
///
/// 使用 HashMap 通过 kid 进行 O(1) 查找，替代线性搜索
fn verify_access_token(
    token: &str,
    jwks: &[Jwk],
    expected_issuer: &str,
) -> Result<AccessTokenClaims> {
    // 从 token 中提取 kid
    let kid = match jwt::extract_kid(token) {
        Ok(Some(k)) => k,
        Ok(None) => {
            tracing::warn!("Token missing kid header");
            return Err(AppError::InvalidToken {
                reason: Some("Token missing kid header".to_string()),
            });
        }
        Err(e) => {
            tracing::warn!("Failed to extract kid from token: {}", e);
            return Err(AppError::InvalidToken {
                reason: Some("Invalid token format".to_string()),
            });
        }
    };

    // 构建 kid -> JWK 的 HashMap 用于 O(1) 查找
    let jwk_map: HashMap<&str, &Jwk> = jwks
        .iter()
        .map(|jwk| (jwk.kid.as_str(), jwk))
        .collect();

    // 通过 kid 查找对应的 JWK
    let jwk = match jwk_map.get(kid.as_str()) {
        Some(jwk) => jwk,
        None => {
            return Err(AppError::InvalidToken {
                reason: Some("Unknown key ID".to_string()),
            });
        }
    };

    // 从 JWK JSON 转换为 PEM 格式的公钥
    let public_key_pem = match KeyService::jwk_to_public_key_pem(&jwk.public_key_jwk) {
        Ok(pem) => pem,
        Err(_) => {
            return Err(AppError::InvalidToken {
                reason: Some("Invalid key format".to_string()),
            });
        }
    };

    // 验证 token
    let token_data = match jwt::verify_jwt::<AccessTokenClaims>(
        token,
        &public_key_pem,
        Some(expected_issuer),
        None, // audience 由客户端控制，不在此验证
    ) {
        Ok(data) => data,
        Err(_) => {
            return Err(AppError::InvalidToken {
                reason: Some("Invalid or expired access token".to_string()),
            });
        }
    };

    // 检查 token 类型
    if token_data.claims.token_type != "oauth-access-token" {
        return Err(AppError::InvalidToken {
            reason: Some("Invalid token type".to_string()),
        });
    }

    Ok(token_data.claims)
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
/// 同时清除浏览器 session cookie 和服务端会话
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(req): Query<LogoutRequest>,
) -> Result<impl IntoResponse> {
    use axum::http::StatusCode;

    // 尝试提取并清除当前 session
    let _session_cleared = if let Some(session_id) = extract_session_from_headers(&headers, &state.config.session_secret) {
        match AuthService::logout(&state.db, &session_id).await {
            Ok(()) => {
                tracing::info!("OIDC logout: session destroyed for {}", &session_id[..8.min(session_id.len())]);
                true
            }
            Err(e) => {
                tracing::warn!("OIDC logout: failed to destroy session: {}", e);
                false
            }
        }
    } else {
        false
    };

    if let Some(ref hint) = req.id_token_hint {
        if !hint.is_empty() {
            let hint_result = LogoutService::validate_id_token_hint::<serde_json::Value>(
                &state.db,
                hint,
                Some(&state.config.issuer),
            )
            .await;

            if hint_result.is_err() {
                return Err(AppError::InvalidToken {
                    reason: Some("Invalid id_token_hint".to_string()),
                });
            }
        }
    }

    let base = &state.config.app_base_url;

    // 验证state参数长度（防止URL注入和请求头过大）
    if let Some(ref s) = req.state {
        const MAX_STATE_LENGTH: usize = 1024;
        if s.len() > MAX_STATE_LENGTH {
            return Err(AppError::InvalidRequest(
                "state parameter exceeds maximum length".to_string(),
            ));
        }
        // 验证state只包含安全的URL字符
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.~".contains(c)) {
            return Err(AppError::InvalidRequest(
                "state parameter contains invalid characters".to_string(),
            ));
        }
    }

    let location = if let Some(ref redirect) = req.post_logout_redirect_uri {
        if redirect.is_empty() {
            return Err(AppError::InvalidRequest(
                "post_logout_redirect_uri cannot be empty".to_string(),
            ));
        }

        let mut validated = false;

        if let Some(ref hint) = req.id_token_hint {
            if !hint.is_empty() {
                if let Ok(claims) = LogoutService::validate_id_token_hint::<serde_json::Value>(
                    &state.db,
                    hint,
                    Some(&state.config.issuer),
                )
                .await {
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

    // 构建 session cookie 清除头
    let secure = state.config.issuer.starts_with("https://");
    let secure_flag = if secure { "; Secure" } else { "" };
    let same_site = if secure { "Strict" } else { "Lax" };
    let mut cookie_value = format!(
        "unoidc_session=; HttpOnly{}; SameSite={}; Path=/; Max-Age=0",
        secure_flag, same_site
    );
    if secure {
        if let Some(domain) = &state.config.cookie_domain {
            cookie_value = format!("{}; Domain={}", cookie_value, domain);
        }
    }

    let mut response = (
        StatusCode::FOUND,
        [(axum::http::header::LOCATION, final_location)],
    ).into_response();
    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie_value).unwrap_or_else(|_| {
            axum::http::HeaderValue::from_static("unoidc_session=; HttpOnly; Path=/; Max-Age=0")
        }),
    );

    Ok::<_, AppError>(response)
}

/// 从请求头中提取 session_id 并验证签名
fn extract_session_from_headers(headers: &HeaderMap, session_secret: &str) -> Option<String> {
    crate::middleware::auth::extract_session_cookie(headers, session_secret)
}

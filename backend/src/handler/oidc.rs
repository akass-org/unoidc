use axum::Json;
use serde_json::{json, Value};
use crate::error::Result;

pub async fn discovery() -> Result<Json<Value>> {
    // TODO: 从配置中读取 issuer
    Ok(Json(json!({
        "issuer": "http://localhost:3000",
        "authorization_endpoint": "http://localhost:3000/authorize",
        "token_endpoint": "http://localhost:3000/token",
        "userinfo_endpoint": "http://localhost:3000/userinfo",
        "jwks_uri": "http://localhost:3000/jwks.json",
        "end_session_endpoint": "http://localhost:3000/logout",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "token_endpoint_auth_methods_supported": ["client_secret_post", "client_secret_basic", "none"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "profile", "email", "groups", "offline_access"],
        "id_token_signing_alg_values_supported": ["ES256"],
        "subject_types_supported": ["public"],
    })))
}

pub async fn jwks() -> Result<Json<Value>> {
    // TODO: 从数据库读取活跃密钥
    Ok(Json(json!({
        "keys": []
    })))
}

pub async fn authorize() -> Result<&'static str> {
    // TODO: 实现授权流程
    Ok("authorize")
}

pub async fn token() -> Result<Json<Value>> {
    // TODO: 实现 token 交换
    Ok(Json(json!({})))
}

pub async fn userinfo() -> Result<Json<Value>> {
    // TODO: 实现 userinfo
    Ok(Json(json!({})))
}

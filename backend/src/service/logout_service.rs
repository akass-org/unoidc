// Logout Service
//
// 处理 RP-Initiated Logout 和会话撤销逻辑

use sqlx::PgPool;
use tracing::{info, warn};

use crate::{error::AppError, model::Client};
use crate::repo::{ClientRepo, RefreshTokenRepo, SessionRepo};
use crate::service::KeyService;
use crate::crypto::jwt;

pub struct LogoutService;

impl LogoutService {
    /// 通过 session_id 登出（删除会话）
    ///
    /// 幂等操作：如果会话不存在也返回成功
    pub async fn logout_by_session(pool: &PgPool, session_id: &str) -> Result<(), AppError> {
        SessionRepo::delete(pool, session_id)
            .await
            .map_err(|e| {
                info!("Failed to delete session during logout: {}", e);
                AppError::InternalServerError { error_code: None }
            })?;

        info!("Session logged out via session_id: {}", session_id);
        Ok(())
    }

    /// 撤销用户所有刷新令牌（但不删除会话）
    pub async fn revoke_all_user_tokens(pool: &PgPool, user_id: uuid::Uuid) -> Result<u64, AppError> {
        RefreshTokenRepo::revoke_all_for_user(pool, user_id)
            .await
            .map_err(|e| {
                info!("Failed to revoke user tokens: {}", e);
                AppError::InternalServerError { error_code: None }
            })
    }

    /// 验证 post_logout_redirect_uri 是否在客户端允许列表中
    ///
    /// 如果客户端未配置 post_logout_redirect_uris，则拒绝所有
    pub async fn validate_post_logout_redirect(
        pool: &PgPool,
        client_id: &uuid::Uuid,
        post_logout_redirect_uri: &str,
    ) -> Result<(), AppError> {
        let client = ClientRepo::find_by_id(pool, *client_id)
            .await
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DATABASE_ERROR: {}", e)),
            })?
            .ok_or_else(|| AppError::ClientNotFound {
                client_id: Some(client_id.to_string()),
            })?;

        let allowed_uris = client.get_post_logout_redirect_uris()
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("PARSE_ERROR: {}", e)),
            })?;

        if allowed_uris.is_empty() {
            return Err(AppError::InvalidRequest(
                "post_logout_redirect_uri not allowed for this client".to_string(),
            ));
        }

        if !allowed_uris.contains(&post_logout_redirect_uri.to_string()) {
            return Err(AppError::InvalidRequest(
                "post_logout_redirect_uri is not in the allowed list".to_string(),
            ));
        }

        Ok(())
    }

    /// 验证 id_token_hint
    ///
    /// 解析 JWT 并验证签名（如果可能），返回其中的 sub claim
    /// 如果 token 格式无效或验证失败，返回错误
    pub async fn validate_id_token_hint<T: serde::de::DeserializeOwned>(
        pool: &PgPool,
        id_token_hint: &str,
    ) -> Result<T, AppError> {
        if id_token_hint.is_empty() {
            return Err(AppError::InvalidRequest(
                "id_token_hint is required".to_string(),
            ));
        }

        let parts: Vec<&str> = id_token_hint.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::InvalidToken {
                reason: Some("Invalid JWT format".to_string()),
            });
        }

        let jwks = KeyService::get_jwks(pool)
            .await
            .map_err(|e| {
                warn!("Failed to get JWKS for id_token_hint validation: {}", e);
                AppError::InternalServerError {
                    error_code: Some("JWKS_ERROR".to_string()),
                }
            })?;

        if jwks.is_empty() {
            return Err(AppError::InternalServerError {
                error_code: Some("No signing keys available".to_string()),
            });
        }

        for jwk in &jwks {
            let public_key_pem = match KeyService::jwk_to_public_key_pem(&jwk.public_key_jwk) {
                Ok(pem) => pem,
                Err(_) => continue,
            };

            match jwt::verify_jwt_no_validate::<T>(id_token_hint, &public_key_pem) {
                Ok(token_data) => {
                    info!("id_token_hint signature verified with kid={}", jwk.kid);
                    return Ok(token_data.claims);
                }
                Err(e) => {
                    warn!("id_token_hint verification attempt failed with kid={}: {}", jwk.kid, e);
                    continue;
                }
            }
        }

        Err(AppError::InvalidToken {
            reason: Some("id_token_hint signature verification failed".to_string()),
        })
    }

    /// 生成 Front-Channel Logout URL
    ///
    /// 根据 OIDC Front-Channel Logout 规范构建回调 URL
    pub fn get_front_channel_logout_uri(client: &Client, session_id: &str) -> Result<String, AppError> {
        let front_channel_uri = client.front_channel_logout_uri
            .as_ref()
            .ok_or_else(|| AppError::InvalidRequest(
                "Client does not support front-channel logout".to_string(),
            ))?;

        // 根据是否已有查询参数选择分隔符
        let separator = if front_channel_uri.contains('?') { '&' } else { '?' };
        Ok(format!(
            "{}{}issuer={}&session_id={}",
            front_channel_uri,
            separator,
            urlencoding::encode(&client.client_id),
            urlencoding::encode(session_id)
        ))
    }
}

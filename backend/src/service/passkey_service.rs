// Passkey Service
//
// WebAuthn 业务逻辑层

use std::sync::Arc;

use sha2::{Digest, Sha256};
use time::Duration;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::{
    error::AppError,
    model::{CreatePasskeyCredential, CreateWebauthnChallenge},
    repo::{PasskeyRepo, UserRepo, WebauthnChallengeRepo},
    AppState,
};

use webauthn_rs::prelude::Credential;

pub struct PasskeyService;

impl PasskeyService {
    /// 开始注册 passkey
    pub async fn start_registration(
        state: &Arc<AppState>,
        user_id: Uuid,
        username: &str,
        display_name: &str,
    ) -> Result<CreationChallengeResponse, AppError> {
        // 获取用户已有的凭据，用于 excludeCredentials
        let existing = PasskeyRepo::list_by_user_id(&state.db, user_id).await?;
        let exclude_credentials = if existing.is_empty() {
            None
        } else {
            Some(
                existing
                    .into_iter()
                    .map(|c| {
                        let bytes: Vec<u8> = c.id.into();
                        CredentialID::from(bytes)
                    })
                    .collect(),
            )
        };

        let (ccr, skr) = state
            .webauthn
            .start_passkey_registration(user_id, username, display_name, exclude_credentials)
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("WEBAUTHN_START_REG: {}", e)),
            })?;

        // 提取 challenge bytes 并计算 hash
        let challenge_bytes: Vec<u8> = ccr.public_key.challenge.clone().into();
        let challenge_hash = Sha256::digest(&challenge_bytes).to_vec();

        // 序列化 registration state
        let state_data = serde_json::to_vec(&skr).map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("SERIALIZE_STATE: {}", e)),
        })?;

        // 存储 challenge
        let expires_at = time::OffsetDateTime::now_utc() + Duration::minutes(5);
        WebauthnChallengeRepo::create(
            &state.db,
            CreateWebauthnChallenge {
                challenge_hash,
                user_id: Some(user_id),
                purpose: "register".to_string(),
                state_data,
                expires_at,
            },
        )
        .await?;

        Ok(ccr)
    }

    /// 完成注册 passkey
    pub async fn finish_registration(
        state: &Arc<AppState>,
        user_id: Uuid,
        reg: &RegisterPublicKeyCredential,
    ) -> Result<(), AppError> {
        // 从 client_data_json 中提取 challenge
        #[derive(serde::Deserialize)]
        struct ClientData {
            challenge: Base64UrlSafeData,
        }

        let client_data: ClientData =
            serde_json::from_slice(reg.response.client_data_json.as_ref())
                .map_err(|e| AppError::InvalidRequest(format!("无法解析 client data: {}", e)))?;

        let challenge_bytes: Vec<u8> = client_data.challenge.into();
        let challenge_hash = Sha256::digest(&challenge_bytes).to_vec();

        // 查找 challenge
        let challenge = WebauthnChallengeRepo::find_by_hash(&state.db, &challenge_hash)
            .await?
            .ok_or_else(|| AppError::InvalidRequest("无效的或已过期的挑战".to_string()))?;

        if challenge.expires_at < time::OffsetDateTime::now_utc() {
            WebauthnChallengeRepo::delete_by_hash(&state.db, &challenge_hash).await?;
            return Err(AppError::InvalidRequest("挑战已过期".to_string()));
        }

        // 恢复 registration state
        let skr: PasskeyRegistration =
            serde_json::from_slice(&challenge.state_data).map_err(|e| {
                AppError::InternalServerError {
                    error_code: Some(format!("DESERIALIZE_STATE: {}", e)),
                }
            })?;

        // 验证注册
        let passkey = state
            .webauthn
            .finish_passkey_registration(reg, &skr)
            .map_err(|e| AppError::InvalidRequest(format!("Passkey 注册验证失败: {}", e)))?;

        // 提取凭据内部信息（danger-credential-internals feature 提供 Into<Credential>）
        let cred: Credential = passkey.into();
        let cred_id: Vec<u8> = cred.cred_id.clone().into();
        let cred_id_str =
            base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &cred_id);
        let public_key =
            serde_json::to_vec(&cred.cred).map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("SERIALIZE_PUBKEY: {}", e)),
            })?;
        let counter = cred.counter as i64;
        let device_type = if cred.backup_eligible {
            Some("multiDevice".to_string())
        } else {
            Some("singleDevice".to_string())
        };
        let backed_up = Some(cred.backup_state);
        let transports = cred.transports.as_ref().map(|t| {
            t.iter()
                .map(|tr| {
                    serde_json::to_string(tr)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string()
                })
                .collect()
        });

        // 检查是否已存在相同 cred_id
        if PasskeyRepo::find_by_id(&state.db, &cred_id_str)
            .await?
            .is_some()
        {
            return Err(AppError::InvalidRequest(
                "该凭据已绑定到您的账户".to_string(),
            ));
        }

        // 存储凭据
        PasskeyRepo::create(
            &state.db,
            CreatePasskeyCredential {
                id: cred_id_str,
                user_id,
                public_key,
                counter,
                device_type,
                backed_up,
                transports,
                display_name: None,
            },
        )
        .await?;

        // 删除已使用的 challenge
        WebauthnChallengeRepo::delete_by_hash(&state.db, &challenge_hash).await?;

        Ok(())
    }

    /// 开始认证 passkey
    pub async fn start_authentication(
        state: &Arc<AppState>,
    ) -> Result<RequestChallengeResponse, AppError> {
        let credentials = PasskeyRepo::list_all(&state.db).await?;

        let passkeys: Vec<Passkey> = credentials
            .into_iter()
            .filter_map(|c| {
                let pk: Credential = serde_json::from_slice(&c.public_key).ok()?;
                Some(Passkey::from(pk))
            })
            .collect();

        let (rcr, skr) = state
            .webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("WEBAUTHN_START_AUTH: {}", e)),
            })?;

        let challenge_bytes: Vec<u8> = rcr.public_key.challenge.clone().into();
        let challenge_hash = Sha256::digest(&challenge_bytes).to_vec();

        let state_data = serde_json::to_vec(&skr).map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("SERIALIZE_STATE: {}", e)),
        })?;

        let expires_at = time::OffsetDateTime::now_utc() + Duration::minutes(5);
        WebauthnChallengeRepo::create(
            &state.db,
            CreateWebauthnChallenge {
                challenge_hash,
                user_id: None,
                purpose: "login".to_string(),
                state_data,
                expires_at,
            },
        )
        .await?;

        Ok(rcr)
    }

    /// 完成认证 passkey
    pub async fn finish_authentication(
        state: &Arc<AppState>,
        auth: &PublicKeyCredential,
    ) -> Result<(crate::model::User, u32), AppError> {
        #[derive(serde::Deserialize)]
        struct ClientData {
            challenge: Base64UrlSafeData,
        }

        let client_data: ClientData =
            serde_json::from_slice(auth.response.client_data_json.as_ref())
                .map_err(|e| AppError::InvalidRequest(format!("无法解析 client data: {}", e)))?;

        let challenge_bytes: Vec<u8> = client_data.challenge.into();
        let challenge_hash = Sha256::digest(&challenge_bytes).to_vec();

        let challenge = WebauthnChallengeRepo::find_by_hash(&state.db, &challenge_hash)
            .await?
            .ok_or_else(|| AppError::InvalidRequest("无效的或已过期的挑战".to_string()))?;

        if challenge.expires_at < time::OffsetDateTime::now_utc() {
            WebauthnChallengeRepo::delete_by_hash(&state.db, &challenge_hash).await?;
            return Err(AppError::InvalidRequest("挑战已过期".to_string()));
        }

        let skr: PasskeyAuthentication =
            serde_json::from_slice(&challenge.state_data).map_err(|e| {
                AppError::InternalServerError {
                    error_code: Some(format!("DESERIALIZE_STATE: {}", e)),
                }
            })?;

        let result = state
            .webauthn
            .finish_passkey_authentication(auth, &skr)
            .map_err(|e| AppError::InvalidRequest(format!("Passkey 认证验证失败: {}", e)))?;

        let cred_id_bytes: Vec<u8> = auth.raw_id.clone().into();
        let cred_id_str =
            base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &cred_id_bytes);

        let cred = PasskeyRepo::find_by_id(&state.db, &cred_id_str)
            .await?
            .ok_or_else(|| AppError::InvalidRequest("未找到匹配的凭据".to_string()))?;

        if result.counter() <= cred.counter as u32 {
            let _ = crate::service::AuditService::log_login_failure(
                &state.db,
                &cred_id_str,
                "passkey_counter_replay",
                None,
                None,
                None,
            )
            .await;
            return Err(AppError::AuthenticationFailed {
                details: Some("检测到安全异常，请尝试密码登录".to_string()),
            });
        }

        PasskeyRepo::update_counter_and_last_used(&state.db, &cred_id_str, result.counter() as i64)
            .await?;

        WebauthnChallengeRepo::delete_by_hash(&state.db, &challenge_hash).await?;

        let user = UserRepo::find_by_id(&state.db, cred.user_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError {
                    error_code: Some("PASSKEY_USER_MISSING".to_string()),
                }
            })?;

        if user.is_locked() || !user.enabled {
            return Err(AppError::Forbidden {
                reason: Some("Account is locked or disabled".to_string()),
            });
        }

        Ok((user, result.counter()))
    }

    /// 列出用户的所有 passkey
    pub async fn list_credentials(
        state: &Arc<AppState>,
        user_id: Uuid,
    ) -> Result<Vec<crate::model::PasskeyCredential>, AppError> {
        PasskeyRepo::list_by_user_id(&state.db, user_id)
            .await
            .map_err(Into::into)
    }

    /// 删除凭据
    pub async fn delete_credential(
        state: &Arc<AppState>,
        id: &str,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let rows = PasskeyRepo::delete(&state.db, id, user_id).await?;
        if rows == 0 {
            return Err(AppError::BusinessError {
                code: "NOT_FOUND".to_string(),
                message: "凭据不存在".to_string(),
            });
        }
        Ok(())
    }
}

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
    model::{CreateGroup, CreatePasskeyCredential, CreateUser, CreateWebauthnChallenge},
    repo::{GroupRepo, PasskeyRepo, UserRepo, WebauthnChallengeRepo},
    AppState,
};
use tracing::{warn};

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
            .map_err(|e| {
                warn!(user_id = %user_id, error = %e, "WebAuthn start_passkey_registration failed");
                AppError::InternalServerError {
                    error_code: Some(format!("WEBAUTHN_START_REG: {}", e)),
                }
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
            .ok_or_else(|| {
                warn!("Challenge not found or expired for passkey registration");
                AppError::InvalidRequest("无效的或已过期的挑战".to_string())
            })?;

        if challenge.expires_at < time::OffsetDateTime::now_utc() {
            warn!("Challenge expired for passkey registration");
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
            .map_err(|e| {
                warn!(user_id = %user_id, error = %e, "WebAuthn finish_passkey_registration failed");
                AppError::InvalidRequest(format!("Passkey 注册验证失败: {}", e))
            })?;

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
            .map_err(|e| {
                warn!(error = %e, "WebAuthn start_passkey_authentication failed");
                AppError::InternalServerError {
                    error_code: Some(format!("WEBAUTHN_START_AUTH: {}", e)),
                }
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
            .ok_or_else(|| {
                warn!("Challenge not found or expired for passkey authentication");
                AppError::InvalidRequest("无效的或已过期的挑战".to_string())
            })?;

        if challenge.expires_at < time::OffsetDateTime::now_utc() {
            warn!("Challenge expired for passkey authentication");
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
            .map_err(|e| {
                warn!(error = %e, "WebAuthn finish_passkey_authentication failed");
                AppError::InvalidRequest(format!("Passkey 认证验证失败: {}", e))
            })?;

        let cred_id_bytes: Vec<u8> = auth.raw_id.clone().into();
        let cred_id_str =
            base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &cred_id_bytes);

        let cred = PasskeyRepo::find_by_id(&state.db, &cred_id_str)
            .await?
            .ok_or_else(|| {
                warn!(credential_id = %cred_id_str, "Credential not found for passkey authentication");
                AppError::InvalidRequest("未找到匹配的凭据".to_string())
            })?;

        if result.counter() <= cred.counter as u32 {
            warn!(credential_id = %cred_id_str, counter = %result.counter(), stored_counter = %cred.counter, "Passkey counter replay detected");
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
            warn!(user_id = %user.id, locked = user.is_locked(), enabled = user.enabled, "Passkey login rejected: account locked or disabled");
            return Err(AppError::Forbidden {
                reason: Some("Account is locked or disabled".to_string()),
            });
        }

        Ok((user, result.counter()))
    }

    /// 开始匿名 passkey 注册（用于纯 passkey 注册）
    pub async fn start_anon_registration(
        state: &Arc<AppState>,
        temp_user_id: Uuid,
        username: &str,
        display_name: &str,
    ) -> Result<CreationChallengeResponse, AppError> {
        let (ccr, skr) = state
            .webauthn
            .start_passkey_registration(temp_user_id, username, display_name, None)
            .map_err(|e| {
                warn!(temp_user_id = %temp_user_id, error = %e, "WebAuthn start_anon_registration failed");
                AppError::InternalServerError {
                    error_code: Some(format!("WEBAUTHN_START_REG: {}", e)),
                }
            })?;

        let challenge_bytes: Vec<u8> = ccr.public_key.challenge.clone().into();
        let challenge_hash = Sha256::digest(&challenge_bytes).to_vec();

        let state_data = serde_json::to_vec(&skr).map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("SERIALIZE_STATE: {}", e)),
        })?;

        let expires_at = time::OffsetDateTime::now_utc() + Duration::minutes(5);
        WebauthnChallengeRepo::create(
            &state.db,
            CreateWebauthnChallenge {
                challenge_hash,
                user_id: Some(temp_user_id),
                purpose: "register_anon".to_string(),
                state_data,
                expires_at,
            },
        )
        .await?;

        Ok(ccr)
    }

    /// 完成匿名 passkey 注册（原子创建用户 + 凭据）
    pub async fn finish_anon_registration(
        state: &Arc<AppState>,
        reg: &RegisterPublicKeyCredential,
        username: String,
        email: String,
        display_name: String,
    ) -> Result<(), AppError> {
        #[derive(serde::Deserialize)]
        struct ClientData {
            challenge: Base64UrlSafeData,
        }

        let client_data: ClientData =
            serde_json::from_slice(reg.response.client_data_json.as_ref())
                .map_err(|e| AppError::InvalidRequest(format!("无法解析 client data: {}", e)))?;

        let challenge_bytes: Vec<u8> = client_data.challenge.into();
        let challenge_hash = Sha256::digest(&challenge_bytes).to_vec();

        let challenge = WebauthnChallengeRepo::find_by_hash(&state.db, &challenge_hash)
            .await?
            .ok_or_else(|| {
                warn!("Challenge not found or expired for anonymous passkey registration");
                AppError::InvalidRequest("无效的或已过期的挑战".to_string())
            })?;

        if challenge.expires_at < time::OffsetDateTime::now_utc() {
            warn!("Challenge expired for anonymous passkey registration");
            WebauthnChallengeRepo::delete_by_hash(&state.db, &challenge_hash).await?;
            return Err(AppError::InvalidRequest("挑战已过期".to_string()));
        }

        if challenge.purpose != "register_anon" {
            warn!(purpose = %challenge.purpose, "Invalid challenge purpose for anonymous passkey registration");
            return Err(AppError::InvalidRequest("无效的挑战用途".to_string()));
        }

        let skr: PasskeyRegistration =
            serde_json::from_slice(&challenge.state_data).map_err(|e| AppError::InternalServerError {
                error_code: Some(format!("DESERIALIZE_STATE: {}", e)),
            })?;

        let passkey = state
            .webauthn
            .finish_passkey_registration(reg, &skr)
            .map_err(|e| {
                warn!(error = %e, "WebAuthn finish_passkey_registration failed for anonymous registration");
                AppError::InvalidRequest(format!("Passkey 注册验证失败: {}", e))
            })?;

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

        if PasskeyRepo::find_by_id(&state.db, &cred_id_str).await?.is_some() {
            warn!(credential_id = %cred_id_str, "Credential already registered for anonymous passkey registration");
            return Err(AppError::InvalidRequest("该凭据已绑定到您的账户".to_string()));
        }

        // 先进行用户名/邮箱唯一性校验（在事务外）
        if UserRepo::find_by_username(&state.db, &username).await?.is_some() {
            warn!(username = %username, "Username already exists for anonymous passkey registration");
            return Err(AppError::BusinessError {
                code: "USER_EXISTS".to_string(),
                message: "用户名已存在".to_string(),
            });
        }
        if UserRepo::find_by_email(&state.db, &email).await?.is_some() {
            warn!(email = %email, "Email already exists for anonymous passkey registration");
            return Err(AppError::BusinessError {
                code: "EMAIL_EXISTS".to_string(),
                message: "邮箱已存在".to_string(),
            });
        }

        let is_first_user = UserRepo::count(&state.db).await? == 0;

        // 原子事务：创建用户 + 存储凭据 + 删除挑战
        let mut tx = state.db.begin().await.map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("TX_BEGIN: {}", e)),
        })?;

        let display_name_for_user = if display_name.is_empty() {
            None
        } else {
            Some(display_name)
        };

        let user = UserRepo::create_in_tx(
            &mut *tx,
            CreateUser {
                username: username.clone(),
                email: email.clone(),
                password_hash: None,
                display_name: display_name_for_user.clone(),
                given_name: None,
                family_name: None,
            },
        )
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint().is_some() {
                    return AppError::BusinessError {
                        code: "USER_EXISTS".to_string(),
                        message: "用户名或邮箱已存在".to_string(),
                    };
                }
            }
            AppError::InternalServerError {
                error_code: Some(format!("USER_CREATE: {}", e)),
            }
        })?;

        // 管理员自动判定逻辑
        if is_first_user {
            let admin_group = match GroupRepo::find_by_name_in_tx(&mut *tx, "admin").await {
                Ok(Some(g)) => g,
                Ok(None) => {
                    GroupRepo::create_in_tx(
                        &mut *tx,
                        CreateGroup {
                            name: "admin".to_string(),
                            description: Some("System administrators".to_string()),
                        },
                    )
                    .await
                    .map_err(|e| AppError::InternalServerError {
                        error_code: Some(format!("ADMIN_GROUP_CREATE: {}", e)),
                    })?
                }
                Err(e) => return Err(AppError::InternalServerError {
                    error_code: Some(format!("ADMIN_GROUP_LOOKUP: {}", e)),
                }),
            };
            GroupRepo::add_user_to_group_in_tx(&mut *tx, user.id, admin_group.id)
                .await
                .map_err(|e| AppError::InternalServerError {
                    error_code: Some(format!("ADMIN_GROUP_ASSIGN: {}", e)),
                })?;
        }

        PasskeyRepo::create_in_tx(
            &mut *tx,
            CreatePasskeyCredential {
                id: cred_id_str,
                user_id: user.id,
                public_key,
                counter,
                device_type,
                backed_up,
                transports,
                display_name: display_name_for_user,
            },
        )
        .await
        .map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("PASSKEY_CREATE: {}", e)),
        })?;

        WebauthnChallengeRepo::delete_by_hash_in_tx(&mut *tx, &challenge_hash)
            .await
            .ok();

        tx.commit().await.map_err(|e| AppError::InternalServerError {
            error_code: Some(format!("TX_COMMIT: {}", e)),
        })?;

        Ok(())
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

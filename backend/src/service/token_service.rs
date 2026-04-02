// Token Service
//
// Token 签发业务逻辑

use anyhow::Result;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::config::Config;
use crate::crypto::{self, jwt};
use crate::metrics;
use crate::model::{AuthorizationCode, Client, CreateRefreshToken, RefreshToken, User};
use crate::repo::{GroupRepo, RefreshTokenRepo, UserRepo};
use crate::service::KeyService;

/// Token 响应
#[derive(Debug, serde::Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub id_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub scope: String,
}

pub struct TokenService;

impl TokenService {
    /// 为授权码签发 tokens
    pub async fn issue_tokens_for_auth_code(
        pool: &PgPool,
        config: &Config,
        auth_code: &AuthorizationCode,
        client: &Client,
    ) -> Result<TokenResponse> {
        // 加载用户
        let user = UserRepo::find_by_id(pool, auth_code.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // 加载用户组
        let groups: Vec<String> = GroupRepo::find_user_groups(pool, user.id)
            .await?
            .into_iter()
            .map(|g| g.name)
            .collect();

        // 获取签名密钥（解密后使用）
        let jwk = KeyService::get_active_key(pool, &config.private_key_encryption_key).await?;

        // 签发 access_token
        let access_token = Self::create_access_token(config, &jwk.kid, &jwk.private_key_pem, &user, client, &auth_code.scope)?;

        // 签发 id_token
        let id_token = Self::create_id_token(config, &jwk.kid, &jwk.private_key_pem, &user, client, auth_code, &groups)?;

        // 签发 refresh_token（如果 scope 包含 offline_access）
        let refresh_token = if auth_code.get_scopes().contains(&"offline_access") {
            Some(Self::create_refresh_token(pool, config, user.id, client.id, &auth_code.scope).await?)
        } else {
            None
        };

        // 更新 token 发放指标
        metrics::TOKEN_ISSUED_TOTAL.inc();
        if refresh_token.is_some() {
            metrics::SESSION_CREATED_TOTAL.inc();
        }

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_token_ttl as u64,
            id_token,
            refresh_token,
            scope: auth_code.scope.clone(),
        })
    }

    /// 使用 refresh_token 签发新 tokens
    pub async fn issue_tokens_for_refresh(
        pool: &PgPool,
        config: &Config,
        plain_refresh_token: &str,
        client: &Client,
    ) -> Result<TokenResponse> {
        let token_hash = crypto::hash_token(plain_refresh_token);

        // 查找 refresh token
        let stored_token = RefreshTokenRepo::find_by_hash(pool, &token_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid refresh token"))?;

        // 验证 client
        if stored_token.client_id != client.id {
            return Err(anyhow::anyhow!("Client mismatch"));
        }

        // 重放检测：如果 token 已被替换，说明有人用新 token 续签过，此 token 的重用就是重放攻击
        if stored_token.is_replaced() {
            tracing::warn!(
                "Refresh token replay detected for user {} client {}",
                stored_token.user_id, client.id
            );
            // 更新重放检测指标
            metrics::REPLAY_DETECTED_TOTAL.inc();
            RefreshTokenRepo::revoke_user_client_tokens(
                pool, stored_token.user_id, client.id,
            ).await?;
            return Err(anyhow::anyhow!("Replay detected - tokens revoked"));
        }

        // 族检测：递归检查整个 token 链，防止使用多代前的旧 token
        if RefreshTokenRepo::detect_family_replay(pool, &token_hash).await? {
            tracing::warn!(
                "Refresh token family replay detected for user {} client {}",
                stored_token.user_id, client.id
            );
            metrics::REPLAY_DETECTED_TOTAL.inc();
            RefreshTokenRepo::revoke_user_client_tokens(
                pool, stored_token.user_id, client.id,
            ).await?;
            return Err(anyhow::anyhow!("Token family replay detected - all tokens revoked"));
        }

        // 标准有效性检查（过期/撤销）
        if !stored_token.is_valid() {
            return Err(anyhow::anyhow!("Refresh token invalid or expired"));
        }

        // 加载用户
        let user = UserRepo::find_by_id(pool, stored_token.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // 检查用户是否仍允许登录
        if !user.can_login() {
            return Err(anyhow::anyhow!("User account is disabled or locked"));
        }

        // 加载用户组
        let groups: Vec<String> = GroupRepo::find_user_groups(pool, user.id)
            .await?
            .into_iter()
            .map(|g| g.name)
            .collect();

        // 获取签名密钥（解密后使用）
        let jwk = KeyService::get_active_key(pool, &config.private_key_encryption_key).await?;

        // 签发新 access_token
        let access_token = Self::create_access_token(config, &jwk.kid, &jwk.private_key_pem, &user, client, &stored_token.scope)?;

        // 签发新 id_token
        let auth_time = stored_token.created_at;
        let id_token = Self::create_id_token_from_refresh(config, &jwk.kid, &jwk.private_key_pem, &user, client, &stored_token.scope, auth_time, &groups)?;

        // 轮换 refresh token
        let new_refresh_token = Self::rotate_refresh_token(pool, config, &stored_token).await?;

        // 更新 token 刷新指标
        metrics::TOKEN_REFRESH_TOTAL.inc();

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_token_ttl as u64,
            id_token,
            refresh_token: Some(new_refresh_token),
            scope: stored_token.scope.clone(),
        })
    }

    /// 创建 access token
    fn create_access_token(
        config: &Config,
        kid: &str,
        private_key_pem: &str,
        user: &User,
        client: &Client,
        scope: &str,
    ) -> Result<String> {
        let now = jwt::now_timestamp();
        let claims = jwt::AccessTokenClaims {
            iss: config.issuer.clone(),
            sub: user.id.to_string(),
            aud: client.client_id.clone(),
            iat: now,
            exp: now + config.access_token_ttl as u64,
            jti: jwt::generate_jti()?,
            scope: scope.to_string(),
            token_type: "oauth-access-token".to_string(),
        };
        jwt::sign_jwt(&claims, kid, private_key_pem)
    }

    /// 创建 id token
    fn create_id_token(
        config: &Config,
        kid: &str,
        private_key_pem: &str,
        user: &User,
        client: &Client,
        auth_code: &AuthorizationCode,
        groups: &[String],
    ) -> Result<String> {
        let now = jwt::now_timestamp();
        let scopes: Vec<&str> = auth_code.scope.split_whitespace().collect();

        let claims = jwt::IdTokenClaims {
            iss: config.issuer.clone(),
            sub: user.id.to_string(),
            aud: client.client_id.clone(),
            iat: now,
            exp: now + config.id_token_ttl as u64,
            jti: jwt::generate_jti()?,
            auth_time: auth_code.auth_time.unix_timestamp() as u64,
            amr: auth_code.get_amr().unwrap_or_default(),
            token_type: "id-token".to_string(),
            nonce: auth_code.nonce.clone(),
            name: if scopes.contains(&"profile") { Some(user.get_display_name().to_string()) } else { None },
            given_name: if scopes.contains(&"profile") { user.given_name.clone() } else { None },
            family_name: if scopes.contains(&"profile") { user.family_name.clone() } else { None },
            preferred_username: if scopes.contains(&"profile") { Some(user.username.clone()) } else { None },
            display_name: if scopes.contains(&"profile") { user.display_name.clone() } else { None },
            picture: if scopes.contains(&"profile") { user.picture.clone() } else { None },
            email: if scopes.contains(&"email") { Some(user.email.clone()) } else { None },
            email_verified: if scopes.contains(&"email") { Some(user.email_verified) } else { None },
            groups: if scopes.contains(&"groups") { Some(groups.to_vec()) } else { None },
        };
        jwt::sign_jwt(&claims, kid, private_key_pem)
    }

    /// 从 refresh token 创建 id token
    fn create_id_token_from_refresh(
        config: &Config,
        kid: &str,
        private_key_pem: &str,
        user: &User,
        client: &Client,
        scope: &str,
        auth_time: OffsetDateTime,
        groups: &[String],
    ) -> Result<String> {
        let now = jwt::now_timestamp();
        let scopes: Vec<&str> = scope.split_whitespace().collect();

        let claims = jwt::IdTokenClaims {
            iss: config.issuer.clone(),
            sub: user.id.to_string(),
            aud: client.client_id.clone(),
            iat: now,
            exp: now + config.id_token_ttl as u64,
            jti: jwt::generate_jti()?,
            auth_time: auth_time.unix_timestamp() as u64,
            amr: vec!["pwd".to_string()],
            token_type: "id-token".to_string(),
            nonce: None,
            name: if scopes.contains(&"profile") { Some(user.get_display_name().to_string()) } else { None },
            given_name: if scopes.contains(&"profile") { user.given_name.clone() } else { None },
            family_name: if scopes.contains(&"profile") { user.family_name.clone() } else { None },
            preferred_username: if scopes.contains(&"profile") { Some(user.username.clone()) } else { None },
            display_name: if scopes.contains(&"profile") { user.display_name.clone() } else { None },
            picture: if scopes.contains(&"profile") { user.picture.clone() } else { None },
            email: if scopes.contains(&"email") { Some(user.email.clone()) } else { None },
            email_verified: if scopes.contains(&"email") { Some(user.email_verified) } else { None },
            groups: if scopes.contains(&"groups") { Some(groups.to_vec()) } else { None },
        };
        jwt::sign_jwt(&claims, kid, private_key_pem)
    }

    /// 创建 refresh token
    async fn create_refresh_token(
        pool: &PgPool,
        config: &Config,
        user_id: uuid::Uuid,
        client_id: uuid::Uuid,
        scope: &str,
    ) -> Result<String> {
        let plain_token = crypto::generate_refresh_token()?;
        let token_hash = crypto::hash_token(&plain_token);
        let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(config.refresh_token_ttl);

        RefreshTokenRepo::create(pool, CreateRefreshToken {
            token_hash,
            parent_token_hash: None,
            user_id,
            client_id,
            scope: scope.to_string(),
            expires_at,
        }).await?;

        Ok(plain_token)
    }

    /// 轮换 refresh token
    async fn rotate_refresh_token(
        pool: &PgPool,
        config: &Config,
        old_token: &RefreshToken,
    ) -> Result<String> {
        let plain_token = crypto::generate_refresh_token()?;
        let new_hash = crypto::hash_token(&plain_token);
        let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(config.refresh_token_ttl);

        // 创建新 token
        RefreshTokenRepo::create(pool, CreateRefreshToken {
            token_hash: new_hash.clone(),
            parent_token_hash: Some(old_token.token_hash.clone()),
            user_id: old_token.user_id,
            client_id: old_token.client_id,
            scope: old_token.scope.clone(),
            expires_at,
        }).await?;

        // 标记旧 token 已替换
        RefreshTokenRepo::mark_replaced(pool, &old_token.token_hash, &new_hash).await?;
        RefreshTokenRepo::update_last_used(pool, &old_token.token_hash).await?;

        Ok(plain_token)
    }
}

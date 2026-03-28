// Consent Service
//
// 用户授权记录业务逻辑层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{Consent, CreateConsent};
use crate::repo::{ClientRepo, ConsentRepo, RefreshTokenRepo, UserRepo};

pub struct ConsentService;

impl ConsentService {
    /// 创建或更新用户授权
    pub async fn grant_consent(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
        scope: String,
    ) -> Result<Consent, anyhow::Error> {
        // 验证用户存在
        UserRepo::find_by_id(pool, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // 验证客户端存在
        ClientRepo::find_by_id(pool, client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // 创建或更新授权
        let consent = ConsentRepo::create(
            pool,
            CreateConsent {
                user_id,
                client_id,
                scope,
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to grant consent: {}", e))?;

        Ok(consent)
    }

    /// 撤销用户授权
    pub async fn revoke_consent(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        // 撤销授权
        ConsentRepo::revoke(pool, user_id, client_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to revoke consent: {}", e))?;

        // 同时撤销该客户端的所有刷新令牌
        RefreshTokenRepo::revoke_user_client_tokens(pool, user_id, client_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to revoke refresh tokens: {}", e))?;

        Ok(())
    }

    /// 检查用户是否已授权客户端
    pub async fn is_authorized(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        ConsentRepo::is_authorized(pool, user_id, client_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check consent: {}", e))
    }

    /// 获取用户的所有授权记录
    pub async fn get_user_consents(pool: &PgPool, user_id: Uuid) -> Result<Vec<Consent>, anyhow::Error> {
        ConsentRepo::find_user_consents(pool, user_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get user consents: {}", e))
    }

    /// 获取用户对特定客户端的授权记录
    pub async fn get_consent(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<Consent>, anyhow::Error> {
        ConsentRepo::find_by_user_and_client(pool, user_id, client_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get consent: {}", e))
    }
}

// Client Service
//
// OIDC 客户端业务逻辑层

use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto;
use crate::model::{Client, CreateClient, UpdateClient};
use crate::repo::{ClientRepo, GroupRepo};

pub struct ClientService;

impl ClientService {
    /// 创建新客户端
    pub async fn create_client(pool: &PgPool, mut input: CreateClient) -> Result<(Client, Option<String>), anyhow::Error> {
        // 验证客户端名称
        if input.name.is_empty() || input.name.len() > 128 {
            return Err(anyhow::anyhow!("Client name must be 1-128 characters"));
        }

        // 验证重定向 URI
        if input.redirect_uris.is_empty() {
            return Err(anyhow::anyhow!("At least one redirect URI is required"));
        }

        // 生成 client_id
        input.client_id = crypto::generate_client_id()?;

        // 如果是机密客户端，生成 client_secret
        let plain_secret = if !input.is_public {
            let secret = crypto::generate_client_secret()?;
            input.client_secret_hash = Some(crypto::hash_client_secret(&secret)?);
            Some(secret)
        } else {
            None
        };

        let client = ClientRepo::create(pool, input)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?;

        Ok((client, plain_secret))
    }

    /// 根据 ID 获取客户端
    pub async fn get_client(pool: &PgPool, id: Uuid) -> Result<Client, anyhow::Error> {
        ClientRepo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))
    }

    /// 根据 client_id 获取客户端
    pub async fn get_client_by_client_id(pool: &PgPool, client_id: &str) -> Result<Client, anyhow::Error> {
        ClientRepo::find_by_client_id(pool, client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))
    }

    /// 获取所有客户端
    pub async fn list_clients(pool: &PgPool) -> Result<Vec<Client>, anyhow::Error> {
        ClientRepo::find_all(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list clients: {}", e))
    }

    /// 获取所有启用的客户端
    pub async fn list_enabled_clients(pool: &PgPool) -> Result<Vec<Client>, anyhow::Error> {
        ClientRepo::find_all_enabled(pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list enabled clients: {}", e))
    }

    /// 更新客户端
    pub async fn update_client(
        pool: &PgPool,
        id: Uuid,
        input: UpdateClient,
    ) -> Result<Client, anyhow::Error> {
        ClientRepo::update(pool, id, input)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update client: {}", e))
    }

    /// 删除客户端
    pub async fn delete_client(pool: &PgPool, id: Uuid) -> Result<(), anyhow::Error> {
        ClientRepo::delete(pool, id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete client: {}", e))
    }

    /// 重新生成客户端密钥
    pub async fn regenerate_secret(pool: &PgPool, id: Uuid) -> Result<String, anyhow::Error> {
        let client = Self::get_client(pool, id).await?;

        if client.is_public {
            return Err(anyhow::anyhow!("Cannot regenerate secret for public client"));
        }

        let plain_secret = crypto::generate_client_secret()?;
        let secret_hash = crypto::hash_client_secret(&plain_secret)?;

        ClientRepo::update_secret(pool, id, secret_hash)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update client secret: {}", e))?;

        Ok(plain_secret)
    }

    /// 验证客户端凭据
    pub async fn verify_client(
        pool: &PgPool,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<Client, anyhow::Error> {
        let client = ClientRepo::find_by_client_id(pool, client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if !client.enabled {
            return Err(anyhow::anyhow!("Client is disabled"));
        }

        // 公开客户端不需要密钥
        if client.is_public {
            return Ok(client);
        }

        // 机密客户端需要验证密钥
        let secret = client_secret
            .ok_or_else(|| anyhow::anyhow!("Client secret required"))?;

        let secret_hash = client
            .client_secret_hash
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client secret not configured"))?;

        if !crypto::verify_client_secret(secret_hash, secret)? {
            return Err(anyhow::anyhow!("Invalid client secret"));
        }

        Ok(client)
    }

    /// 添加客户端到组
    pub async fn add_client_to_group(
        pool: &PgPool,
        client_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        // 验证客户端存在
        ClientRepo::find_by_id(pool, client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // 验证组存在
        GroupRepo::find_by_id(pool, group_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        ClientRepo::add_client_to_group(pool, client_id, group_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add client to group: {}", e))
    }

    /// 从组中移除客户端
    pub async fn remove_client_from_group(
        pool: &PgPool,
        client_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        ClientRepo::remove_client_from_group(pool, client_id, group_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove client from group: {}", e))
    }

    /// 检查用户是否可以访问客户端
    pub async fn can_user_access_client(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        ClientRepo::can_user_access_client(pool, user_id, client_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check client access: {}", e))
    }
}

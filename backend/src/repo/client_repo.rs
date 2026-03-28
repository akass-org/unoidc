// Client Repository
//
// OIDC 客户端数据访问层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{Client, CreateClient, UpdateClient};

pub struct ClientRepo;

impl ClientRepo {
    /// 根据ID查找客户端
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Client>, sqlx::Error> {
        sqlx::query_as::<_, Client>(
            r#"
            SELECT * FROM clients WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 根据 client_id 查找客户端
    pub async fn find_by_client_id(pool: &PgPool, client_id: &str) -> Result<Option<Client>, sqlx::Error> {
        sqlx::query_as::<_, Client>(
            r#"
            SELECT * FROM clients WHERE client_id = $1
            "#,
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await
    }

    /// 获取所有客户端
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Client>, sqlx::Error> {
        sqlx::query_as::<_, Client>(
            r#"
            SELECT * FROM clients ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// 获取所有启用的客户端
    pub async fn find_all_enabled(pool: &PgPool) -> Result<Vec<Client>, sqlx::Error> {
        sqlx::query_as::<_, Client>(
            r#"
            SELECT * FROM clients WHERE enabled = true ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// 创建客户端
    pub async fn create(pool: &PgPool, input: CreateClient) -> Result<Client, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();
        let redirect_uris = serde_json::to_value(&input.redirect_uris).unwrap();
        let post_logout_redirect_uris = input.post_logout_redirect_uris
            .map(|uris| serde_json::to_value(&uris).unwrap());
        let grant_types = serde_json::to_value(&input.grant_types).unwrap();
        let response_types = serde_json::to_value(&input.response_types).unwrap();

        sqlx::query_as::<_, Client>(
            r#"
            INSERT INTO clients (
                id, client_id, client_secret_hash, is_public, name, description,
                app_url, redirect_uris, post_logout_redirect_uris, grant_types,
                response_types, token_endpoint_auth_method, id_token_signed_response_alg,
                enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.client_id)
        .bind(&input.client_secret_hash)
        .bind(input.is_public)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.app_url)
        .bind(redirect_uris)
        .bind(post_logout_redirect_uris)
        .bind(grant_types)
        .bind(response_types)
        .bind(&input.token_endpoint_auth_method)
        .bind("ES256")
        .bind(true)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 更新客户端
    pub async fn update(pool: &PgPool, id: Uuid, input: UpdateClient) -> Result<Client, sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();
        let mut client = Self::find_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)?;

        if let Some(name) = input.name {
            client.name = name;
        }
        if let Some(description) = input.description {
            client.description = Some(description);
        }
        if let Some(app_url) = input.app_url {
            client.app_url = Some(app_url);
        }
        if let Some(redirect_uris) = input.redirect_uris {
            client.redirect_uris = serde_json::to_value(&redirect_uris).unwrap();
        }
        if let Some(post_logout_redirect_uris) = input.post_logout_redirect_uris {
            client.post_logout_redirect_uris = Some(serde_json::to_value(&post_logout_redirect_uris).unwrap());
        }
        if let Some(enabled) = input.enabled {
            client.enabled = enabled;
        }
        client.updated_at = now;

        sqlx::query_as::<_, Client>(
            r#"
            UPDATE clients
            SET
                name = $2,
                description = $3,
                app_url = $4,
                redirect_uris = $5,
                post_logout_redirect_uris = $6,
                enabled = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&client.name)
        .bind(&client.description)
        .bind(&client.app_url)
        .bind(&client.redirect_uris)
        .bind(&client.post_logout_redirect_uris)
        .bind(client.enabled)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 删除客户端
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM clients WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 更新客户端密钥
    pub async fn update_secret(pool: &PgPool, id: Uuid, secret_hash: String) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE clients
            SET client_secret_hash = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(secret_hash)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 添加客户端到组
    pub async fn add_client_to_group(pool: &PgPool, client_id: Uuid, group_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO client_groups (client_id, group_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(client_id)
        .bind(group_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 从组中移除客户端
    pub async fn remove_client_from_group(pool: &PgPool, client_id: Uuid, group_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM client_groups
            WHERE client_id = $1 AND group_id = $2
            "#,
        )
        .bind(client_id)
        .bind(group_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 获取客户端可访问的组列表
    pub async fn find_client_groups(pool: &PgPool, client_id: Uuid) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
        let results: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT group_id FROM client_groups WHERE client_id = $1
            "#,
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;

        Ok(results.into_iter().map(|r| r.0).collect())
    }

    /// 检查用户是否可以访问客户端（通过组关系）
    pub async fn can_user_access_client(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM user_groups ug
            INNER JOIN client_groups cg ON ug.group_id = cg.group_id
            WHERE ug.user_id = $1 AND cg.client_id = $2
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}

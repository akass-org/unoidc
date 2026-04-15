// Passkey Credential Repository
//
// Passkey 凭据数据访问层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{CreatePasskeyCredential, PasskeyCredential};

pub struct PasskeyRepo;

impl PasskeyRepo {
    /// 根据用户ID列出所有凭据
    pub async fn list_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<PasskeyCredential>, sqlx::Error> {
        sqlx::query_as::<_, PasskeyCredential>(
            r#"
            SELECT * FROM passkey_credentials
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// 列出所有凭据
    pub async fn list_all(pool: &PgPool) -> Result<Vec<PasskeyCredential>, sqlx::Error> {
        sqlx::query_as::<_, PasskeyCredential>(
            "SELECT * FROM passkey_credentials ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// 根据凭据ID查找
    pub async fn find_by_id(
        pool: &PgPool,
        id: &str,
    ) -> Result<Option<PasskeyCredential>, sqlx::Error> {
        sqlx::query_as::<_, PasskeyCredential>(
            r#"
            SELECT * FROM passkey_credentials WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 创建凭据
    pub async fn create(
        pool: &PgPool,
        input: CreatePasskeyCredential,
    ) -> Result<PasskeyCredential, sqlx::Error> {
        sqlx::query_as::<_, PasskeyCredential>(
            r#"
            INSERT INTO passkey_credentials (
                id, user_id, public_key, counter, device_type, backed_up,
                transports, display_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&input.id)
        .bind(input.user_id)
        .bind(&input.public_key)
        .bind(input.counter)
        .bind(&input.device_type)
        .bind(input.backed_up)
        .bind(&input.transports)
        .bind(&input.display_name)
        .fetch_one(pool)
        .await
    }

    /// 删除指定用户的指定凭据
    pub async fn delete(pool: &PgPool, id: &str, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM passkey_credentials WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 更新签名计数器和最后使用时间
    pub async fn update_counter_and_last_used(
        pool: &PgPool,
        id: &str,
        counter: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE passkey_credentials
            SET counter = $2, last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(counter)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

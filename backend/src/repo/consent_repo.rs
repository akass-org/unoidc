// Consent Repository
//
// 用户授权记录数据访问层

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{Consent, CreateConsent};

pub struct ConsentRepo;

impl ConsentRepo {
    /// 根据ID查找授权记录
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Consent>, sqlx::Error> {
        sqlx::query_as::<_, Consent>(
            r#"
            SELECT * FROM user_consents WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 查找用户对客户端的授权记录
    pub async fn find_by_user_and_client(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<Consent>, sqlx::Error> {
        sqlx::query_as::<_, Consent>(
            r#"
            SELECT * FROM user_consents
            WHERE user_id = $1 AND client_id = $2
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .fetch_optional(pool)
        .await
    }

    /// 获取用户的所有授权记录
    pub async fn find_user_consents(pool: &PgPool, user_id: Uuid) -> Result<Vec<Consent>, sqlx::Error> {
        sqlx::query_as::<_, Consent>(
            r#"
            SELECT * FROM user_consents
            WHERE user_id = $1 AND revoked_at IS NULL
            ORDER BY granted_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// 创建授权记录
    pub async fn create(pool: &PgPool, input: CreateConsent) -> Result<Consent, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();

        sqlx::query_as::<_, Consent>(
            r#"
            INSERT INTO user_consents (id, user_id, client_id, scope, granted_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, client_id)
            DO UPDATE SET
                scope = EXCLUDED.scope,
                revoked_at = NULL,
                updated_at = EXCLUDED.updated_at
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.user_id)
        .bind(input.client_id)
        .bind(&input.scope)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 撤销授权
    pub async fn revoke(pool: &PgPool, user_id: Uuid, client_id: Uuid) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE user_consents
            SET revoked_at = $3, updated_at = $3
            WHERE user_id = $1 AND client_id = $2
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 查找用户和客户端的授权记录（包括已撤销的最新记录）
    pub async fn find_revoked_consent(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<Consent>, sqlx::Error> {
        sqlx::query_as::<_, Consent>(
            r#"
            SELECT * FROM user_consents
            WHERE user_id = $1 AND client_id = $2
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .fetch_optional(pool)
        .await
    }

    /// 检查用户是否已授权客户端
    pub async fn is_authorized(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM user_consents
            WHERE user_id = $1
              AND client_id = $2
              AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}

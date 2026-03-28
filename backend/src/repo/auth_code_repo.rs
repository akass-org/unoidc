// Authorization Code Repository
//
// 授权码数据访问层

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{AuthorizationCode, CreateAuthorizationCode};

pub struct AuthCodeRepo;

impl AuthCodeRepo {
    /// 根据授权码哈希查找
    pub async fn find_by_hash(pool: &PgPool, code_hash: &str) -> Result<Option<AuthorizationCode>, sqlx::Error> {
        sqlx::query_as::<_, AuthorizationCode>(
            r#"
            SELECT * FROM authorization_codes WHERE code_hash = $1
            "#,
        )
        .bind(code_hash)
        .fetch_optional(pool)
        .await
    }

    /// 创建授权码
    pub async fn create(pool: &PgPool, input: CreateAuthorizationCode) -> Result<AuthorizationCode, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let expires_at = now + time::Duration::minutes(10); // 授权码 10 分钟有效
        let amr = serde_json::to_value(&input.amr).unwrap();

        sqlx::query_as::<_, AuthorizationCode>(
            r#"
            INSERT INTO authorization_codes (
                id, code_hash, user_id, client_id, redirect_uri, scope, nonce,
                code_challenge, code_challenge_method, auth_time, amr, expires_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.code_hash)
        .bind(input.user_id)
        .bind(input.client_id)
        .bind(&input.redirect_uri)
        .bind(&input.scope)
        .bind(&input.nonce)
        .bind(&input.code_challenge)
        .bind(&input.code_challenge_method)
        .bind(input.auth_time)
        .bind(amr)
        .bind(expires_at)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 标记授权码为已使用
    pub async fn consume(pool: &PgPool, code_hash: &str) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE authorization_codes
            SET consumed_at = $2
            WHERE code_hash = $1
            "#,
        )
        .bind(code_hash)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 清理过期的授权码
    pub async fn delete_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            DELETE FROM authorization_codes WHERE expires_at < $1
            "#,
        )
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

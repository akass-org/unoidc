// JWK Repository
//
// JSON Web Key 签名密钥数据访问层

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{CreateJwk, Jwk};

pub struct JwkRepo;

impl JwkRepo {
    /// 根据ID查找密钥
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Jwk>, sqlx::Error> {
        sqlx::query_as::<_, Jwk>(
            r#"
            SELECT * FROM jwks WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// 根据 kid 查找密钥
    pub async fn find_by_kid(pool: &PgPool, kid: &str) -> Result<Option<Jwk>, sqlx::Error> {
        sqlx::query_as::<_, Jwk>(
            r#"
            SELECT * FROM jwks WHERE kid = $1
            "#,
        )
        .bind(kid)
        .fetch_optional(pool)
        .await
    }

    /// 获取当前激活的密钥
    pub async fn find_active(pool: &PgPool) -> Result<Option<Jwk>, sqlx::Error> {
        sqlx::query_as::<_, Jwk>(
            r#"
            SELECT * FROM jwks
            WHERE active = true
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await
    }

    /// 获取所有密钥（用于 JWKS 端点）
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Jwk>, sqlx::Error> {
        sqlx::query_as::<_, Jwk>(
            r#"
            SELECT * FROM jwks ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// 创建新密钥
    pub async fn create(pool: &PgPool, input: CreateJwk) -> Result<Jwk, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Jwk>(
            r#"
            INSERT INTO jwks (id, kid, alg, kty, private_key_pem, public_key_jwk, active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.kid)
        .bind(&input.alg)
        .bind(&input.kty)
        .bind(&input.private_key_pem)
        .bind(&input.public_key_jwk)
        .bind(true) // 默认激活
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 激活密钥（同时停用其他密钥）
    pub async fn activate(pool: &PgPool, id: Uuid) -> Result<Jwk, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        // 先停用所有密钥
        sqlx::query(
            r#"
            UPDATE jwks SET active = false
            "#,
        )
        .execute(pool)
        .await?;

        // 然后激活指定密钥
        let jwk = sqlx::query_as::<_, Jwk>(
            r#"
            UPDATE jwks
            SET active = true, rotated_at = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(jwk)
    }

    /// 删除密钥
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM jwks WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

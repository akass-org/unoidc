// Refresh Token Repository
//
// 刷新令牌数据访问层

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::model::{CreateRefreshToken, RefreshToken};

pub struct RefreshTokenRepo;

impl RefreshTokenRepo {
    /// 根据令牌哈希查找（带行锁，用于防止竞态条件）
    pub async fn find_by_hash_for_update(pool: &PgPool, token_hash: &str) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM refresh_tokens WHERE token_hash = $1 FOR UPDATE
            "#,
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    /// 根据令牌哈希查找
    pub async fn find_by_hash(pool: &PgPool, token_hash: &str) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM refresh_tokens WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    /// 创建刷新令牌
    pub async fn create(pool: &PgPool, input: CreateRefreshToken) -> Result<RefreshToken, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (
                id, token_hash, parent_token_hash, user_id, client_id, scope,
                expires_at, created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&input.token_hash)
        .bind(&input.parent_token_hash)
        .bind(input.user_id)
        .bind(input.client_id)
        .bind(&input.scope)
        .bind(input.expires_at)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    /// 撤销令牌
    pub async fn revoke(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = $2
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 撤销用户对客户端的所有刷新令牌
    pub async fn revoke_user_client_tokens(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = $3
            WHERE user_id = $1 AND client_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(client_id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 标记令牌已被替换（轮换）
    pub async fn mark_replaced(
        pool: &PgPool,
        old_token_hash: &str,
        new_token_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET replaced_by_token_hash = $2
            WHERE token_hash = $1
            "#,
        )
        .bind(old_token_hash)
        .bind(new_token_hash)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 更新最后使用时间
    pub async fn update_last_used(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET last_used_at = $2
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 清理过期的刷新令牌
    pub async fn delete_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens WHERE expires_at < $1
            "#,
        )
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 检测令牌重放攻击（父令牌被重用）
    pub async fn detect_replay(pool: &PgPool, parent_token_hash: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM refresh_tokens
            WHERE parent_token_hash = $1 AND revoked_at IS NOT NULL
            "#,
        )
        .bind(parent_token_hash)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    /// 递归检测 token 族重放攻击
    ///
    /// 只要发现当前 token 的祖先出现“撤销”或“分叉替换”就判定为重放风险
    /// 防止攻击者使用多代之前的旧 token 进行重放
    pub async fn detect_family_replay(pool: &PgPool, token_hash: &str) -> Result<bool, sqlx::Error> {
        // 递归 CTE 查询：追踪整个 token 族谱
        let result: (i64,) = sqlx::query_as(
            r#"
            WITH RECURSIVE token_chain AS (
                -- 起始 token
                SELECT token_hash, parent_token_hash, replaced_by_token_hash, revoked_at
                FROM refresh_tokens
                WHERE token_hash = $1
                UNION ALL
                -- 向上追溯所有祖先
                SELECT rt.token_hash, rt.parent_token_hash, rt.replaced_by_token_hash, rt.revoked_at
                FROM refresh_tokens rt
                INNER JOIN token_chain tc ON rt.token_hash = tc.parent_token_hash
            )
            SELECT COUNT(*)
            FROM token_chain
                        WHERE token_hash != $1
                            AND (
                                revoked_at IS NOT NULL
                                OR replaced_by_token_hash IS NOT NULL
                            )
            "#,
        )
        .bind(token_hash)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    /// 撤销用户的所有刷新令牌
    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = $2
            WHERE user_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

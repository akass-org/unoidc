// Password reset token repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{CreatePasswordResetToken, PasswordResetToken};

pub struct PasswordResetTokenRepo;

impl PasswordResetTokenRepo {
    /// 创建密码重置令牌
    pub async fn create(
        pool: &PgPool,
        input: CreatePasswordResetToken,
    ) -> Result<PasswordResetToken, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, PasswordResetToken>(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.user_id)
        .bind(&input.token_hash)
        .bind(input.expires_at)
        .fetch_one(pool)
        .await
    }

    /// 根据哈希查找令牌（仅未使用的）
    pub async fn find_by_hash(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, sqlx::Error> {
        sqlx::query_as::<_, PasswordResetToken>(
            "SELECT * FROM password_reset_tokens WHERE token_hash = $1 AND consumed_at IS NULL",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    /// 标记令牌已使用
    pub async fn mark_consumed(
        pool: &PgPool,
        token_id: Uuid,
    ) -> Result<Option<PasswordResetToken>, sqlx::Error> {
        sqlx::query_as::<_, PasswordResetToken>(
            "UPDATE password_reset_tokens SET consumed_at = NOW() WHERE id = $1 AND consumed_at IS NULL RETURNING *",
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await
    }

    /// 撤销用户所有待处理的密码重置令牌
    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1 AND consumed_at IS NULL")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 清理过期的令牌
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at < NOW()")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}

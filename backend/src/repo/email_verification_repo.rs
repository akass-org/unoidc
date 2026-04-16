// Email verification token repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::model::{CreateEmailVerificationToken, EmailVerificationToken};

pub struct EmailVerificationTokenRepo;

impl EmailVerificationTokenRepo {
    /// Create a new email verification token
    pub async fn create(
        pool: &PgPool,
        input: CreateEmailVerificationToken,
    ) -> Result<EmailVerificationToken, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, EmailVerificationToken>(
            "INSERT INTO email_verification_tokens 
             (id, user_id, new_email, token_hash, expires_at) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING *",
        )
        .bind(id)
        .bind(input.user_id)
        .bind(input.new_email)
        .bind(input.token_hash)
        .bind(input.expires_at)
        .fetch_one(pool)
        .await
    }

    /// Find token by hash
    pub async fn find_by_hash(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationToken>, sqlx::Error> {
        sqlx::query_as::<_, EmailVerificationToken>(
            "SELECT * FROM email_verification_tokens WHERE token_hash = $1 AND verified_at IS NULL",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
    }

    /// Mark token as verified
    pub async fn mark_verified(
        pool: &PgPool,
        token_id: Uuid,
    ) -> Result<EmailVerificationToken, sqlx::Error> {
        sqlx::query_as::<_, EmailVerificationToken>(
            "UPDATE email_verification_tokens SET verified_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(token_id)
        .fetch_one(pool)
        .await
    }

    /// Revoke all pending tokens for a user
    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM email_verification_tokens WHERE user_id = $1 AND verified_at IS NULL",
        )
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM email_verification_tokens WHERE expires_at < NOW()")
            .execute(pool)
            .await?;
        Ok(result.rows_affected() as i64)
    }
}

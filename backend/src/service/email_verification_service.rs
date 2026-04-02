// Email verification service
//
// Handles email address verification workflow

use anyhow::Result;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::crypto;
use crate::model::{CreateEmailVerificationToken, User};
use crate::repo::EmailVerificationTokenRepo;

const EMAIL_TOKEN_TTL_MINUTES: i64 = 24 * 60; // 24 hours

pub struct EmailVerificationService;

impl EmailVerificationService {
    /// Request email change with verification token
    ///
    /// Returns plain token (for sending to user's new email)
    pub async fn request_email_change(
        pool: &PgPool,
        user: &User,
        new_email: &str,
    ) -> Result<String> {
        // Validate email format
        if !new_email.contains('@') {
            return Err(anyhow::anyhow!("Invalid email format"));
        }

        // Generate token
        let plain_token = crypto::generate_secure_token(32)?;
        let token_hash = crypto::hash_token(&plain_token);

        // Revoke any pending verification tokens for this user
        EmailVerificationTokenRepo::revoke_all_for_user(pool, user.id).await?;

        // Create new verification token
        let expires_at = OffsetDateTime::now_utc() + time::Duration::minutes(EMAIL_TOKEN_TTL_MINUTES);

        EmailVerificationTokenRepo::create(
            pool,
            CreateEmailVerificationToken {
                user_id: user.id,
                new_email: new_email.to_string(),
                token_hash,
                expires_at,
            },
        )
        .await?;

        Ok(plain_token)
    }

    /// Verify email change by token
    ///
    /// Returns new verified email address
    pub async fn verify_email_change(
        pool: &PgPool,
        plain_token: &str,
    ) -> Result<String> {
        let token_hash = crypto::hash_token(plain_token);

        // Find and validate token
        let token = EmailVerificationTokenRepo::find_by_hash(pool, &token_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired verification token"))?;

        if !token.is_valid() {
            return Err(anyhow::anyhow!("Verification token has expired"));
        }

        // Mark as verified
        let verified_token = EmailVerificationTokenRepo::mark_verified(pool, token.id)
            .await?;

        Ok(verified_token.new_email)
    }

    /// Clean up expired tokens (should be called by background job)
    pub async fn cleanup_expired_tokens(pool: &PgPool) -> Result<i64> {
        let count = EmailVerificationTokenRepo::cleanup_expired(pool).await?;
        Ok(count)
    }
}

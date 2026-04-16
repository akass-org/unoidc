// Session cleanup service
//
// Periodically clean up expired sessions and email verification tokens

use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

use crate::repo::{EmailVerificationTokenRepo, SessionRepo, WebauthnChallengeRepo};

pub struct CleanupService;

impl CleanupService {
    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(pool: &PgPool) -> Result<i64> {
        let deleted_count = SessionRepo::delete_expired(pool).await? as i64;
        if deleted_count > 0 {
            info!("Cleaned up {} expired sessions", deleted_count);
        }
        Ok(deleted_count)
    }

    /// Clean up expired email verification tokens
    pub async fn cleanup_expired_email_tokens(pool: &PgPool) -> Result<i64> {
        let deleted_count = EmailVerificationTokenRepo::cleanup_expired(pool).await?;
        if deleted_count > 0 {
            info!(
                "Cleaned up {} expired email verification tokens",
                deleted_count
            );
        }
        Ok(deleted_count)
    }

    /// Clean up expired WebAuthn challenges
    pub async fn cleanup_expired_webauthn_challenges(pool: &PgPool) -> Result<i64> {
        let deleted_count = WebauthnChallengeRepo::delete_expired(pool).await?;
        if deleted_count > 0 {
            info!("Cleaned up {} expired WebAuthn challenges", deleted_count);
        }
        Ok(deleted_count as i64)
    }

    /// Run full cleanup (should be called periodically by background job)
    ///
    /// Example: Call every 1 hour via a scheduled task
    pub async fn run_full_cleanup(pool: &PgPool) -> Result<(i64, i64, i64)> {
        let sessions_cleaned = Self::cleanup_expired_sessions(pool).await?;
        let email_tokens_cleaned = Self::cleanup_expired_email_tokens(pool).await?;
        let challenges_cleaned = Self::cleanup_expired_webauthn_challenges(pool).await?;

        info!(
            "Full cleanup completed: {} sessions, {} email tokens, {} webauthn challenges",
            sessions_cleaned, email_tokens_cleaned, challenges_cleaned
        );

        Ok((sessions_cleaned, email_tokens_cleaned, challenges_cleaned))
    }
}

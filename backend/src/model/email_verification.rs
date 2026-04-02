// Email verification token model

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub new_email: String,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub verified_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl EmailVerificationToken {
    /// 检查 token 是否有效（未过期且未验证）
    pub fn is_valid(&self) -> bool {
        self.verified_at.is_none() && self.expires_at > OffsetDateTime::now_utc()
    }
}

#[derive(Debug, Clone)]
pub struct CreateEmailVerificationToken {
    pub user_id: Uuid,
    pub new_email: String,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
}

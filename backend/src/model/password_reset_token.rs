// Password reset token model

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub consumed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl PasswordResetToken {
    /// 检查 token 是否有效（未过期且未使用）
    pub fn is_valid(&self) -> bool {
        self.consumed_at.is_none() && self.expires_at > OffsetDateTime::now_utc()
    }
}

#[derive(Debug, Clone)]
pub struct CreatePasswordResetToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
}

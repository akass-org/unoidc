// WebAuthn Challenge 数据模型
//
// 对应数据库表: webauthn_challenges

use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct WebauthnChallenge {
    pub challenge_hash: Vec<u8>,
    pub user_id: Option<Uuid>,
    pub purpose: String,
    pub state_data: Vec<u8>,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// 创建 WebauthnChallenge 时的参数
#[derive(Debug, Clone)]
pub struct CreateWebauthnChallenge {
    pub challenge_hash: Vec<u8>,
    pub user_id: Option<Uuid>,
    pub purpose: String,
    pub state_data: Vec<u8>,
    pub expires_at: OffsetDateTime,
}

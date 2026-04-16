// Passkey Credential 数据模型
//
// 对应数据库表: passkey_credentials

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasskeyCredential {
    pub id: String,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    pub public_key: Vec<u8>,
    pub counter: i64,
    pub device_type: Option<String>,
    pub backed_up: Option<bool>,
    pub transports: Option<Vec<String>>,
    pub display_name: Option<String>,
    pub created_at: OffsetDateTime,
    pub last_used_at: Option<OffsetDateTime>,
}

/// 创建 PasskeyCredential 时的参数
#[derive(Debug, Clone)]
pub struct CreatePasskeyCredential {
    pub id: String,
    pub user_id: Uuid,
    pub public_key: Vec<u8>,
    pub counter: i64,
    pub device_type: Option<String>,
    pub backed_up: Option<bool>,
    pub transports: Option<Vec<String>>,
    pub display_name: Option<String>,
}

// JWK 数据模型
//
// 对应数据库表: jwks
// JSON Web Key 签名密钥

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Jwk {
    pub id: Uuid,
    pub kid: String,
    pub alg: String,
    pub kty: String,
    #[serde(skip_serializing)]
    pub private_key_pem: String,
    pub public_key_jwk: serde_json::Value,
    pub active: bool,
    pub created_at: OffsetDateTime,
    pub rotated_at: Option<OffsetDateTime>,
}

impl Jwk {
    /// 检查是否为当前激活的密钥
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 获取公钥 JWK
    pub fn get_public_jwk(&self) -> Result<serde_json::Value, serde_json::Error> {
        Ok(self.public_key_jwk.clone())
    }
}

/// 创建 JWK 时的参数
#[derive(Debug, Clone)]
pub struct CreateJwk {
    pub kid: String,
    pub alg: String,
    pub kty: String,
    pub private_key_pem: String,
    pub public_key_jwk: serde_json::Value,
}

// AuthorizationCode 数据模型
//
// 对应数据库表: authorization_codes
// OIDC 授权码

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthorizationCode {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub code_hash: String,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub scope: String,
    pub nonce: Option<String>,
    #[serde(skip_serializing)]
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub auth_time: OffsetDateTime,
    pub amr: serde_json::Value, // Authentication Methods References (JSON array)
    pub expires_at: OffsetDateTime,
    pub consumed_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl AuthorizationCode {
    /// 检查授权码是否过期
    pub fn is_expired(&self) -> bool {
        self.expires_at <= OffsetDateTime::now_utc()
    }

    /// 检查授权码是否已被使用
    pub fn is_consumed(&self) -> bool {
        self.consumed_at.is_some()
    }

    /// 检查授权码是否有效（未过期且未使用）
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_consumed()
    }

    /// 获取 scope 列表
    pub fn get_scopes(&self) -> Vec<&str> {
        self.scope.split_whitespace().collect()
    }

    /// 获取 AMR 列表
    pub fn get_amr(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.amr.clone())
    }
}

/// 创建授权码时的参数
#[derive(Debug, Clone)]
pub struct CreateAuthorizationCode {
    pub code_hash: String,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub scope: String,
    pub nonce: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub auth_time: OffsetDateTime,
    pub amr: Vec<String>,
}

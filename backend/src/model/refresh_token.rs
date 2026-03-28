// RefreshToken 数据模型
//
// 对应数据库表: refresh_tokens
// OIDC 刷新令牌

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub token_hash: String,
    #[serde(skip_serializing)]
    pub parent_token_hash: Option<String>,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scope: String,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    #[serde(skip_serializing)]
    pub replaced_by_token_hash: Option<String>,
    pub created_at: OffsetDateTime,
    pub last_used_at: Option<OffsetDateTime>,
}

impl RefreshToken {
    /// 检查刷新令牌是否过期
    pub fn is_expired(&self) -> bool {
        self.expires_at <= OffsetDateTime::now_utc()
    }

    /// 检查刷新令牌是否已被撤销
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// 检查刷新令牌是否已被替换
    pub fn is_replaced(&self) -> bool {
        self.replaced_by_token_hash.is_some()
    }

    /// 检查刷新令牌是否有效（未过期、未撤销、未替换）
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked() && !self.is_replaced()
    }

    /// 获取 scope 列表
    pub fn get_scopes(&self) -> Vec<&str> {
        self.scope.split_whitespace().collect()
    }

    /// 检查是否包含指定的 scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.get_scopes().contains(&scope)
    }
}

/// 创建刷新令牌时的参数
#[derive(Debug, Clone)]
pub struct CreateRefreshToken {
    pub token_hash: String,
    pub parent_token_hash: Option<String>,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scope: String,
    pub expires_at: OffsetDateTime,
}

/// 刷新令牌轮换结果
#[derive(Debug, Clone)]
pub struct RefreshTokenRotation {
    pub old_token: RefreshToken,
    pub new_token: RefreshToken,
}

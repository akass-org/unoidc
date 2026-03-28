// Consent 数据模型
//
// 对应数据库表: user_consents
// 用户对客户端的授权记录

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Consent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scope: String,
    pub granted_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub updated_at: OffsetDateTime,
}

impl Consent {
    /// 检查授权是否有效（未被撤销）
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none()
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

/// 创建授权时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConsent {
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scope: String,
}

/// 授权查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentQuery {
    pub user_id: Uuid,
    pub client_id: Uuid,
}

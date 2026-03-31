// Group 数据模型
//
// 对应数据库表: groups

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}

impl Group {
    /// 创建新组
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// 创建组时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroup {
    pub name: String,
    pub description: Option<String>,
}

/// 更新组时的参数
///
/// 使用 `Option<Option<String>>` 模式来区分 "不修改" 和 "清空":
/// - `None` = 不修改该字段
/// - `Some(None)` = 清空（设置为 NULL）
/// - `Some(Some(value))` = 设置为新值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroup {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

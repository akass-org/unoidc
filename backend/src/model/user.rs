// User 数据模型
//
// 对应数据库表: users

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
    pub last_login_at: Option<OffsetDateTime>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl User {
    /// 检查用户是否被锁定
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            locked_until > OffsetDateTime::now_utc()
        } else {
            false
        }
    }

    /// 检查账户是否可以登录
    pub fn can_login(&self) -> bool {
        self.enabled && !self.is_locked()
    }

    /// 获取显示名称（优先使用 display_name，否则使用 username）
    pub fn get_display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }
}

/// 创建用户时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

/// 更新用户时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: Option<bool>,
    pub enabled: Option<bool>,
}

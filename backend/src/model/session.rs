// Session 数据模型
//
// 对应数据库表: user_sessions
// 浏览器会话管理

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: Uuid,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    /// 检查会话是否过期
    pub fn is_expired(&self) -> bool {
        self.expires_at <= OffsetDateTime::now_utc()
    }

    /// 检查会话是否有效
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// 更新最后访问时间
    pub fn touch(&mut self) {
        self.last_seen_at = OffsetDateTime::now_utc();
    }
}

/// 创建会话时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSession {
    pub user_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub duration_seconds: i64,
}

impl CreateSession {
    /// 创建新的会话
    pub fn new(user_id: Uuid, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self {
            user_id,
            ip_address,
            user_agent,
            duration_seconds: 86400, // 默认 24 小时
        }
    }
}

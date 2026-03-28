// AuditLog 数据模型
//
// 对应数据库表: audit_logs
// 审计日志

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub correlation_id: String,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub outcome: String,
    pub reason_code: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}

impl AuditLog {
    /// 检查操作是否成功
    pub fn is_success(&self) -> bool {
        self.outcome == "success"
    }
}

/// 创建审计日志时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLog {
    pub actor_user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub correlation_id: String,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub outcome: String,
    pub reason_code: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl CreateAuditLog {
    /// 创建成功的审计日志
    pub fn success(
        action: impl Into<String>,
        target_type: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_user_id: None,
            client_id: None,
            correlation_id: uuid::Uuid::new_v4().to_string(),
            action: action.into(),
            target_type: target_type.into(),
            target_id: target_id.into(),
            outcome: "success".to_string(),
            reason_code: None,
            metadata: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// 创建失败的审计日志
    pub fn failure(
        action: impl Into<String>,
        target_type: impl Into<String>,
        target_id: impl Into<String>,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            actor_user_id: None,
            client_id: None,
            correlation_id: uuid::Uuid::new_v4().to_string(),
            action: action.into(),
            target_type: target_type.into(),
            target_id: target_id.into(),
            outcome: "failure".to_string(),
            reason_code: Some(reason_code.into()),
            metadata: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// 设置操作用户
    pub fn with_actor(mut self, user_id: Uuid) -> Self {
        self.actor_user_id = Some(user_id);
        self
    }

    /// 设置客户端
    pub fn with_client(mut self, client_id: Uuid) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// 设置关联 ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = correlation_id.into();
        self
    }

    /// 设置 IP 地址
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// 设置 User Agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 审计日志查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogQuery {
    pub actor_user_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub action: Option<String>,
    pub outcome: Option<String>,
    pub from_time: Option<OffsetDateTime>,
    pub to_time: Option<OffsetDateTime>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

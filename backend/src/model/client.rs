// Client 数据模型
//
// 对应数据库表: clients
// OIDC 客户端配置

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Client {
    pub id: Uuid,
    pub client_id: String,
    #[serde(skip_serializing)]
    pub client_secret_hash: Option<String>,
    pub is_public: bool,
    pub name: String,
    pub description: Option<String>,
    pub app_url: Option<String>,
    pub redirect_uris: serde_json::Value, // JSON array
    pub post_logout_redirect_uris: Option<serde_json::Value>, // JSON array
    pub grant_types: serde_json::Value,   // JSON array
    pub response_types: serde_json::Value, // JSON array
    pub token_endpoint_auth_method: String,
    pub id_token_signed_response_alg: String,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Client {
    /// 检查客户端是否为机密客户端（非公开客户端）
    pub fn is_confidential(&self) -> bool {
        !self.is_public
    }

    /// 获取重定向 URI 列表
    pub fn get_redirect_uris(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.redirect_uris.clone())
    }

    /// 获取 Post Logout 重定向 URI 列表
    pub fn get_post_logout_redirect_uris(&self) -> Result<Vec<String>, serde_json::Error> {
        match &self.post_logout_redirect_uris {
            Some(v) => serde_json::from_value(v.clone()),
            None => Ok(vec![]),
        }
    }

    /// 获取授权类型列表
    pub fn get_grant_types(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.grant_types.clone())
    }

    /// 获取响应类型列表
    pub fn get_response_types(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.response_types.clone())
    }

    /// 检查重定向 URI 是否有效
    pub fn is_valid_redirect_uri(&self, uri: &str) -> bool {
        if let Ok(uris) = self.get_redirect_uris() {
            uris.iter().any(|u| u == uri)
        } else {
            false
        }
    }

    /// 检查授权类型是否支持
    pub fn supports_grant_type(&self, grant_type: &str) -> bool {
        if let Ok(grant_types) = self.get_grant_types() {
            grant_types.iter().any(|g| g == grant_type)
        } else {
            false
        }
    }
}

/// 创建客户端时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClient {
    pub client_id: String,
    pub client_secret_hash: Option<String>,
    pub is_public: bool,
    pub name: String,
    pub description: Option<String>,
    pub app_url: Option<String>,
    pub redirect_uris: Vec<String>,
    pub post_logout_redirect_uris: Option<Vec<String>>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub token_endpoint_auth_method: String,
}

/// 更新客户端时的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub description: Option<String>,
    pub app_url: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub post_logout_redirect_uris: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

/// 统一错误类型
///
/// 支持业务错误、认证错误、验证错误和 OIDC 协议错误
#[derive(Debug, Error)]
pub enum AppError {
    /// 数据库错误
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// 配置错误
    #[error("Configuration error: {0}")]
    ConfigError(#[from] anyhow::Error),

    /// 认证失败
    #[error("Authentication failed")]
    AuthenticationFailed {
        /// 错误详情
        details: Option<String>,
    },

    /// 凭据无效
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// 用户未找到
    #[error("User not found")]
    UserNotFound {
        /// 用户标识符（用户名或 ID）
        identifier: Option<String>,
    },

    /// 客户端未找到
    #[error("Client not found")]
    ClientNotFound {
        /// 客户端 ID
        client_id: Option<String>,
    },

    /// 未授权访问
    #[error("Unauthorized")]
    Unauthorized {
        /// 原因说明
        reason: Option<String>,
    },

    /// 禁止访问
    #[error("Forbidden")]
    Forbidden {
        /// 原因说明
        reason: Option<String>,
    },

    /// 请求无效
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// 验证错误
    #[error("Validation error")]
    ValidationError {
        /// 字段名
        field: String,
        /// 错误消息
        message: String,
    },

    /// OIDC 协议错误
    #[error("OIDC error")]
    OidcError {
        /// 错误代码（RFC 6749 定义）
        error: OidcErrorCode,
        /// 错误描述
        error_description: Option<String>,
    },

    /// Token 过期
    #[error("Token expired")]
    TokenExpired {
        /// Token 类型
        token_type: Option<String>,
    },

    /// Token 无效
    #[error("Invalid token")]
    InvalidToken {
        /// 原因说明
        reason: Option<String>,
    },

    /// 速率限制超出
    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        /// 重试时间（秒）
        retry_after: Option<u64>,
    },

    /// 内部服务器错误
    #[error("Internal server error")]
    InternalServerError {
        /// 内部错误代码（用于追踪）
        error_code: Option<String>,
    },

    /// 业务逻辑错误
    #[error("Business error")]
    BusinessError {
        /// 错误代码
        code: String,
        /// 错误消息
        message: String,
    },
}

/// OIDC 标准错误代码（RFC 6749）
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum OidcErrorCode {
    /// 无效请求
    #[error("invalid_request")]
    InvalidRequest,

    /// 未授权客户端
    #[error("unauthorized_client")]
    UnauthorizedClient,

    /// 拒绝访问
    #[error("access_denied")]
    AccessDenied,

    /// 不支持的响应类型
    #[error("unsupported_response_type")]
    UnsupportedResponseType,

    /// 无效的范围
    #[error("invalid_scope")]
    InvalidScope,

    /// 服务器错误
    #[error("server_error")]
    ServerError,

    /// 暂时不可用
    #[error("temporarily_unavailable")]
    TemporarilyUnavailable,

    /// 无效客户端
    #[error("invalid_client")]
    InvalidClient,

    /// 无效授权
    #[error("invalid_grant")]
    InvalidGrant,

    /// 不支持的授权类型
    #[error("unsupported_grant_type")]
    UnsupportedGrantType,

    /// 无效的 Token
    #[error("invalid_token")]
    InvalidToken,
}

fn sanitize_public_error_code(raw: Option<&String>) -> String {
    match raw {
        Some(code) => {
            // 仅暴露稳定前缀（如 DB_ERROR），丢弃冒号后的内部细节。
            let head = code.split(':').next().unwrap_or("INTERNAL_ERROR").trim();
            if head.is_empty() {
                "INTERNAL_ERROR".to_string()
            } else {
                head.to_string()
            }
        }
        None => "INTERNAL_ERROR".to_string(),
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code, details) = match &self {
            // 数据库错误：不暴露内部信息
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
                Some("DATABASE_ERROR".to_string()),
                None,
            ),

            // 配置错误：不暴露内部信息
            AppError::ConfigError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
                Some("CONFIG_ERROR".to_string()),
                None,
            ),

            // 认证失败
            AppError::AuthenticationFailed { details } => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed".to_string(),
                Some("AUTHENTICATION_FAILED".to_string()),
                details.clone(),
            ),

            // 无效凭据
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid credentials".to_string(),
                Some("INVALID_CREDENTIALS".to_string()),
                None,
            ),

            // 用户未找到
            AppError::UserNotFound { identifier } => (
                StatusCode::NOT_FOUND,
                "User not found".to_string(),
                Some("USER_NOT_FOUND".to_string()),
                identifier.as_ref().map(|id| format!("Identifier: {}", id)),
            ),

            // 客户端未找到
            AppError::ClientNotFound { client_id } => (
                StatusCode::NOT_FOUND,
                "Client not found".to_string(),
                Some("CLIENT_NOT_FOUND".to_string()),
                client_id.as_ref().map(|id| format!("Client ID: {}", id)),
            ),

            // 未授权
            AppError::Unauthorized { reason } => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized".to_string(),
                Some("UNAUTHORIZED".to_string()),
                reason.clone(),
            ),

            // 禁止访问
            AppError::Forbidden { reason } => (
                StatusCode::FORBIDDEN,
                "Forbidden".to_string(),
                Some("FORBIDDEN".to_string()),
                reason.clone(),
            ),

            // 无效请求
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                msg.clone(),
                Some("INVALID_REQUEST".to_string()),
                None,
            ),

            // 验证错误
            AppError::ValidationError { field, message } => (
                StatusCode::BAD_REQUEST,
                message.clone(),
                Some("VALIDATION_ERROR".to_string()),
                Some(format!("Field: {}", field)),
            ),

            // OIDC 协议错误：特殊处理
            AppError::OidcError {
                error,
                error_description,
            } => {
                let status = match error {
                    OidcErrorCode::InvalidRequest
                    | OidcErrorCode::InvalidScope
                    | OidcErrorCode::UnsupportedResponseType
                    | OidcErrorCode::UnsupportedGrantType => StatusCode::BAD_REQUEST,

                    OidcErrorCode::InvalidClient
                    | OidcErrorCode::UnauthorizedClient
                    | OidcErrorCode::InvalidGrant
                    | OidcErrorCode::InvalidToken => StatusCode::UNAUTHORIZED,

                    OidcErrorCode::AccessDenied => StatusCode::FORBIDDEN,

                    OidcErrorCode::ServerError
                    | OidcErrorCode::TemporarilyUnavailable => {
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                };

                // OIDC 错误使用标准格式
                return (
                    status,
                    Json(json!({
                        "error": error.to_string(),
                        "error_description": error_description,
                        "status": status.as_u16(),
                    })),
                )
                    .into_response();
            }

            // Token 过期
            AppError::TokenExpired { token_type } => (
                StatusCode::UNAUTHORIZED,
                "Token expired".to_string(),
                Some("TOKEN_EXPIRED".to_string()),
                token_type.clone(),
            ),

            // 无效 Token
            AppError::InvalidToken { reason } => (
                StatusCode::UNAUTHORIZED,
                "Invalid token".to_string(),
                Some("INVALID_TOKEN".to_string()),
                reason.clone(),
            ),

            // 速率限制
            AppError::RateLimitExceeded { retry_after } => {
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "Rate limit exceeded",
                        "error_code": "RATE_LIMIT_EXCEEDED",
                        "status": StatusCode::TOO_MANY_REQUESTS.as_u16(),
                        "details": retry_after.map(|s| format!("Retry after {} seconds", s)),
                    })),
                )
                    .into_response();

                // 添加 Retry-After header
                if let Some(seconds) = retry_after {
                    response.headers_mut().insert(
                        axum::http::header::RETRY_AFTER,
                        axum::http::HeaderValue::from_str(&seconds.to_string()).unwrap(),
                    );
                }

                return response;
            }

            // 内部服务器错误
            AppError::InternalServerError { error_code } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                Some(sanitize_public_error_code(error_code.as_ref())),
                None,
            ),

            // 业务错误
            AppError::BusinessError { code, message } => (
                StatusCode::BAD_REQUEST,
                message.clone(),
                Some(code.clone()),
                None,
            ),
        };

        // 统一错误响应格式
        let body = Json(json!({
            "error": error_message,
            "error_code": error_code,
            "status": status.as_u16(),
            "details": details,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn internal_server_error_strips_sensitive_error_details() {
        let response = AppError::InternalServerError {
            error_code: Some("DB_ERROR: relation users does not exist".to_string()),
        }
        .into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["error_code"], "DB_ERROR");
        assert_eq!(payload["error"], "Internal server error");
    }
}

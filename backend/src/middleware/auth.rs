// Auth 中间件
//
// 提供从请求中提取 session 的 helper 函数

use axum::http::HeaderMap;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::model::{Session, User};
use crate::service::AuthService;

/// 已认证的用户信息
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
    pub session: Session,
}

/// 从请求头中提取 session cookie 并验证
///
/// 成功返回 AuthUser，失败返回 None（不报错，由调用方决定处理方式）
pub async fn extract_auth_user(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<Option<AuthUser>> {
    let session_id = extract_session_cookie(headers);
    let session_id = match session_id {
        Some(id) => id,
        None => return Ok(None),
    };

    match AuthService::validate_session(pool, &session_id).await? {
        Some((user, session)) => Ok(Some(AuthUser { user, session })),
        None => Ok(None),
    }
}

/// 要求必须已认证，否则返回 401
pub async fn require_auth_user(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<AuthUser> {
    extract_auth_user(pool, headers)
        .await?
        .ok_or(AppError::Unauthorized {
            reason: Some("Authentication required".to_string()),
        })
}

/// 从 Cookie 头中提取 unoidc_session
/// 
/// 返回 session_id（不包含签名）
pub fn extract_session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    let cookie_value = extract_cookie_value(cookie_header, "unoidc_session")?;
    
    // Cookie 格式: session_id.signature
    // 只返回 session_id 部分
    let (session_id, _signature) = cookie_value.split_once('.')?;
    Some(session_id.to_string())
}

/// 从 cookie 字符串中提取指定 cookie 的值
///
/// 先 split '=' 再比较 name，避免 `session` 匹配到 `session_id`
fn extract_cookie_value(cookie_str: &str, name: &str) -> Option<String> {
    cookie_str
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            let (cookie_name, value) = cookie.split_once('=')?;
            if cookie_name.trim() == name {
                Some(value.to_string())
            } else {
                None
            }
        })
}

/// 检查用户是否可以访问指定客户端（通过组关系）
pub async fn check_user_client_access(
    pool: &PgPool,
    user_id: Uuid,
    client_db_id: Uuid,
) -> Result<()> {
    let has_access = crate::repo::ClientRepo::can_user_access_client(pool, user_id, client_db_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error while checking client access: {}", e);
            AppError::InternalServerError {
                error_code: Some("ACCESS_CHECK_ERROR".to_string()),
            }
        })?;

    if !has_access {
        return Err(AppError::Forbidden {
            reason: Some("User does not have access to this client".to_string()),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cookie_value_found() {
        let cookies = "session=abc123; unoidc_session=xyz789; theme=dark";
        assert_eq!(
            extract_cookie_value(cookies, "unoidc_session"),
            Some("xyz789".to_string())
        );
    }

    #[test]
    fn test_extract_cookie_value_first() {
        let cookies = "unoidc_session=first";
        assert_eq!(
            extract_cookie_value(cookies, "unoidc_session"),
            Some("first".to_string())
        );
    }

    #[test]
    fn test_extract_cookie_value_not_found() {
        let cookies = "session=abc123; theme=dark";
        assert_eq!(extract_cookie_value(cookies, "unoidc_session"), None);
    }

    #[test]
    fn test_extract_cookie_value_empty() {
        assert_eq!(extract_cookie_value("", "unoidc_session"), None);
    }
}

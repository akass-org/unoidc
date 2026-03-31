// OIDC Service
//
// 授权流程业务逻辑：授权码签发、PKCE 验证、consent 管理

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{error, info};
use uuid::Uuid;

use crate::crypto;
use crate::metrics;
use crate::model::{AuthorizationCode, CreateAuthorizationCode};
use crate::repo::AuthCodeRepo;
use crate::service::ConsentService;

/// 内置 scope 集合
const VALID_SCOPES: &[&str] = &["openid", "profile", "email", "groups", "offline_access"];

pub struct OidcService;

impl OidcService {
    /// 签发授权码
    ///
    /// 生成随机授权码，哈希后存入数据库，返回明文授权码
    pub async fn issue_authorization_code(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
        redirect_uri: &str,
        scope: &str,
        nonce: Option<&str>,
        code_challenge: &str,
        code_challenge_method: &str,
        auth_time: OffsetDateTime,
    ) -> anyhow::Result<(String, AuthorizationCode)> {
        // 生成明文授权码
        let plain_code = crypto::generate_authorization_code()?;

        // 哈希存储
        let code_hash = crypto::hash_token(&plain_code);

        let input = CreateAuthorizationCode {
            code_hash,
            user_id,
            client_id,
            redirect_uri: redirect_uri.to_string(),
            scope: scope.to_string(),
            nonce: nonce.map(|s| s.to_string()),
            code_challenge: code_challenge.to_string(),
            code_challenge_method: code_challenge_method.to_string(),
            auth_time,
            amr: vec!["pwd".to_string()],
        };

        let auth_code = AuthCodeRepo::create(pool, input).await?;

        info!(
            "Issued authorization code for user {} client {}, expires in 10min",
            user_id, client_id
        );

        Ok((plain_code, auth_code))
    }

    /// 通过哈希查找并原子消费授权码
    ///
    /// 使用条件 UPDATE 确保并发安全，防止授权码双花。
    /// 如果检测到重放攻击（code 存在但已被消费），记录安全事件。
    pub async fn exchange_authorization_code(
        pool: &PgPool,
        plain_code: &str,
    ) -> anyhow::Result<Option<AuthorizationCode>> {
        let code_hash = crypto::hash_token(plain_code);

        // 原子操作：仅在未消费时标记并返回
        let auth_code = match AuthCodeRepo::consume_and_return(pool, &code_hash).await? {
            Some(code) => code,
            None => {
                // 区分 "code 不存在" 和 "code 已消费（重放攻击）"
                let code_exists = AuthCodeRepo::exists(pool, &code_hash).await?;
                if code_exists {
                    // 重放攻击检测：code 存在但已被消费
                    metrics::AUTH_CODE_REPLAY_TOTAL.inc();
                    error!(
                        "Security event: authorization code replay detected (code_hash: {})",
                        code_hash
                    );
                    // 注：这里没有调用 AuditService，因为需要 request_context 中的 IP/UA 信息
                    // 调用方应在收到 None 且知道是重放时补充审计日志
                }
                return Ok(None);
            }
        };

        // 检查是否过期（可能在消费窗口内过期）
        if auth_code.is_expired() {
            return Ok(None);
        }

        Ok(Some(auth_code))
    }

    /// 检测授权码重放攻击
    ///
    /// 在 consume_and_return 返回 None 后调用，检查 code 是否存在
    pub async fn is_auth_code_replay(pool: &PgPool, code_hash: &str) -> anyhow::Result<bool> {
        AuthCodeRepo::exists(pool, code_hash).await.map_err(Into::into)
    }

    /// 验证 scope 列表合法性
    ///
    /// 必须包含 openid，所有 scope 必须在内置列表中
    pub fn validate_scopes(scope: &str) -> Result<Vec<&str>, String> {
        let scopes: Vec<&str> = scope.split_whitespace().collect();

        if !scopes.contains(&"openid") {
            return Err("scope must include 'openid'".to_string());
        }

        for s in &scopes {
            if !VALID_SCOPES.contains(s) {
                return Err(format!("unsupported scope: {}", s));
            }
        }

        Ok(scopes)
    }

    /// 检查用户是否已有对客户端的充分授权
    ///
    /// 比较请求的 scope 是否是已授权 scope 的子集
    pub async fn check_consent_coverage(
        pool: &PgPool,
        user_id: Uuid,
        client_id: Uuid,
        requested_scope: &str,
    ) -> anyhow::Result<bool> {
        let consent = ConsentService::get_consent(pool, user_id, client_id).await?;

        match consent {
            Some(consent) if consent.is_valid() => {
                let requested: Vec<&str> = requested_scope.split_whitespace().collect();
                let granted = consent.get_scopes();
                let covered = requested.iter().all(|s| granted.contains(s));
                Ok(covered)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_scopes_valid() {
        let scopes = OidcService::validate_scopes("openid profile email").unwrap();
        assert_eq!(scopes, vec!["openid", "profile", "email"]);
    }

    #[test]
    fn test_validate_scopes_missing_openid() {
        let result = OidcService::validate_scopes("profile email");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("openid"));
    }

    #[test]
    fn test_validate_scopes_unsupported() {
        let result = OidcService::validate_scopes("openid custom_scope");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("custom_scope"));
    }

    #[test]
    fn test_validate_scopes_minimal() {
        let scopes = OidcService::validate_scopes("openid").unwrap();
        assert_eq!(scopes, vec!["openid"]);
    }

    #[test]
    fn test_verify_pkce_s256() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = crypto::hash_token(verifier);

        assert!(crypto::verify_pkce_s256(verifier, &challenge));
        assert!(!crypto::verify_pkce_s256("wrong-verifier", &challenge));
    }

    #[test]
    fn test_hash_token_deterministic() {
        let h1 = crypto::hash_token("my-code");
        let h2 = crypto::hash_token("my-code");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_token_unique() {
        let h1 = crypto::hash_token("code-a");
        let h2 = crypto::hash_token("code-b");
        assert_ne!(h1, h2);
    }
}

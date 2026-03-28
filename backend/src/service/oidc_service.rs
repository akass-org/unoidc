// OIDC Service
//
// 授权流程业务逻辑：授权码签发、PKCE 验证、consent 管理

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::info;
use uuid::Uuid;

use crate::crypto;
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
        let code_hash = Self::hash_token(&plain_code);

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

    /// 通过哈希查找并消费授权码
    ///
    /// 原子操作：查找 → 验证 → 标记已消费
    pub async fn exchange_authorization_code(
        pool: &PgPool,
        plain_code: &str,
    ) -> anyhow::Result<Option<AuthorizationCode>> {
        let code_hash = Self::hash_token(plain_code);

        let auth_code = match AuthCodeRepo::find_by_hash(pool, &code_hash).await? {
            Some(code) => code,
            None => return Ok(None),
        };

        // 检查是否已消费
        if auth_code.consumed_at.is_some() {
            return Ok(None);
        }

        // 检查是否过期
        if auth_code.is_expired() {
            return Ok(None);
        }

        // 标记已消费
        AuthCodeRepo::consume(pool, &code_hash).await?;

        Ok(Some(auth_code))
    }

    /// PKCE S256 验证
    ///
    /// BASE64URL(SHA256(code_verifier)) == code_challenge
    pub fn verify_pkce_s256(code_verifier: &str, code_challenge: &str) -> bool {
        let hash = Sha256::digest(code_verifier.as_bytes());
        let computed = URL_SAFE_NO_PAD.encode(&hash);
        computed == code_challenge
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

    /// 通用 token 哈希（SHA-256 → base64url）
    ///
    /// 用于授权码、refresh token 等的存储哈希
    pub fn hash_token(token: &str) -> String {
        let hash = Sha256::digest(token.as_bytes());
        URL_SAFE_NO_PAD.encode(&hash)
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
        let hash = Sha256::digest(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(&hash);

        assert!(OidcService::verify_pkce_s256(verifier, &challenge));
        assert!(!OidcService::verify_pkce_s256("wrong-verifier", &challenge));
    }

    #[test]
    fn test_hash_token_deterministic() {
        let h1 = OidcService::hash_token("my-code");
        let h2 = OidcService::hash_token("my-code");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_token_unique() {
        let h1 = OidcService::hash_token("code-a");
        let h2 = OidcService::hash_token("code-b");
        assert_ne!(h1, h2);
    }
}

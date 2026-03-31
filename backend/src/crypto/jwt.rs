// JWT 签名与验证工具
//
// 使用 ES256 (P-256 + SHA-256) 算法签发和验证 JWT
// 所有时间字段均为 Unix 时间戳 (NumericDate)，符合 OIDC 规范

use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// ID Token Claims (OIDC Core Section 2)
///
/// 默认包含标准 claims，按 scope 扩展 profile/email/groups 字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub iat: u64,
    pub exp: u64,
    pub jti: String,
    pub auth_time: u64,
    pub amr: Vec<String>,
    #[serde(rename = "type")]
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    // profile scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    // email scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    // groups scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
}

/// Access Token Claims
///
/// 精简结构，仅包含必要的身份和授权信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub iat: u64,
    pub exp: u64,
    pub jti: String,
    pub scope: String,
    #[serde(rename = "type")]
    pub token_type: String,
}

/// 使用 ES256 签发 JWT
pub fn sign_jwt<T: Serialize>(claims: &T, kid: &str, private_key_pem: &str) -> Result<String> {
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(kid.to_string());
    header.typ = Some("JWT".to_string());

    let key = EncodingKey::from_ec_pem(private_key_pem.as_bytes())?;
    let token = encode(&header, claims, &key)?;
    Ok(token)
}

/// 验证并解码 JWT
pub fn verify_jwt<T: DeserializeOwned>(
    token: &str,
    public_key_pem: &str,
    issuer: Option<&str>,
    audience: Option<&str>,
) -> Result<jsonwebtoken::TokenData<T>> {
    let key = DecodingKey::from_ec_pem(public_key_pem.as_bytes())?;
    let mut validation = Validation::new(Algorithm::ES256);

    if let Some(iss) = issuer {
        validation.set_issuer(&[iss]);
    }

    if let Some(aud) = audience {
        validation.set_audience(&[aud]);
    } else {
        validation.validate_aud = false;
    }

    let data = decode::<T>(token, &key, &validation)?;
    Ok(data)
}

#[deprecated(
    since = "0.1.0",
    note = "verify_jwt_no_validate bypasses audience/issuer validation. Use verify_jwt for full validation. Only to be used for internal token verification where the caller performs its own claim validation."
)]
pub fn verify_jwt_no_validate<T: DeserializeOwned>(
    token: &str,
    public_key_pem: &str,
) -> Result<jsonwebtoken::TokenData<T>> {
    let key = DecodingKey::from_ec_pem(public_key_pem.as_bytes())?;
    let mut validation = Validation::new(Algorithm::ES256);
    validation.validate_aud = false;
    validation.set_required_spec_claims(&["exp".to_string()]);
    let data = decode::<T>(token, &key, &validation)?;
    Ok(data)
}

/// 生成唯一的 JWT ID
pub fn generate_jti() -> Result<String> {
    super::generate_secure_token(16)
}

/// 获取当前 Unix 时间戳（秒）
pub fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time before epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意: ES256 密钥对的完整签名/验证测试在 key_service 的集成测试中进行
    // 这里测试 claims 序列化格式
    #[test]
    fn test_id_token_claims_serialization() {
        let claims = IdTokenClaims {
            iss: "https://example.com".to_string(),
            sub: "user-123".to_string(),
            aud: "client-abc".to_string(),
            iat: 1000000,
            exp: 1003600,
            jti: "jti-xyz".to_string(),
            auth_time: 999999,
            amr: vec!["pwd".to_string()],
            token_type: "id-token".to_string(),
            nonce: Some("nonce-123".to_string()),
            name: None,
            given_name: None,
            family_name: None,
            preferred_username: None,
            display_name: None,
            picture: None,
            email: None,
            email_verified: None,
            groups: None,
        };

        let json = serde_json::to_value(&claims).unwrap();
        // 确认时间字段是数字而非字符串
        assert!(json["iat"].is_number());
        assert!(json["exp"].is_number());
        assert!(json["auth_time"].is_number());
        // 确认 None 字段被跳过
        assert!(json.get("name").is_none());
        assert!(json.get("email").is_none());
        assert!(json.get("groups").is_none());
        // 确认 type 字段正确序列化
        assert_eq!(json["type"], "id-token");
        assert_eq!(json["nonce"], "nonce-123");
    }

    #[test]
    fn test_access_token_claims_serialization() {
        let claims = AccessTokenClaims {
            iss: "https://example.com".to_string(),
            sub: "user-123".to_string(),
            aud: "client-abc".to_string(),
            iat: 1000000,
            exp: 1003600,
            jti: "jti-xyz".to_string(),
            scope: "openid profile".to_string(),
            token_type: "oauth-access-token".to_string(),
        };

        let json = serde_json::to_value(&claims).unwrap();
        assert!(json["iat"].is_number());
        assert_eq!(json["type"], "oauth-access-token");
        assert_eq!(json["scope"], "openid profile");
    }
}

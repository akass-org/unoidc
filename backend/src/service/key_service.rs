// Key Service
//
// ES256 签名密钥管理：生成、轮换、JWKS 输出

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::rand_core::OsRng;
use pkcs8::EncodePrivateKey;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::info;

use crate::crypto::{self, key_encryption};
use crate::model::{CreateJwk, Jwk};
use crate::repo::JwkRepo;

pub struct KeyService;

impl KeyService {
    /// 生成新的 P-256 密钥对并存入数据库
    pub async fn generate_key_pair(pool: &PgPool, encryption_key: &str) -> Result<Jwk> {
        let kid = crypto::generate_secure_token(16)?;
        let signing_key = SigningKey::random(&mut OsRng);
        let _verifying_key = signing_key.verifying_key();

        // 导出私钥为 PKCS8 PEM
        let private_key_pem = signing_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("Failed to export private key: {}", e))?
            .to_string();

        // 构造公钥 JWK
        let public_key_jwk = Self::build_public_jwk(&signing_key, &kid);

        // 加密私钥后存储
        let encrypted_pem = key_encryption::encrypt_private_key(&private_key_pem, encryption_key)?;

        let input = CreateJwk {
            kid,
            alg: "ES256".to_string(),
            kty: "EC".to_string(),
            private_key_pem: encrypted_pem,
            public_key_jwk,
        };

        let jwk = JwkRepo::create(pool, input)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store JWK: {}", e))?;

        info!("Generated new ES256 key pair: kid={}", jwk.kid);
        Ok(jwk)
    }

    /// 解密 JWK 中的私钥
    fn decrypt_jwk(jwk: Jwk, encryption_key: &str) -> Result<Jwk> {
        let decrypted_pem = key_encryption::decrypt_private_key(&jwk.private_key_pem, encryption_key)?;
        Ok(Jwk { private_key_pem: decrypted_pem, ..jwk })
    }

    /// 获取当前激活的密钥，不存在则自动生成
    pub async fn get_active_key(pool: &PgPool, encryption_key: &str) -> Result<Jwk> {
        if let Some(jwk) = JwkRepo::find_active(pool).await? {
            return Ok(Self::decrypt_jwk(jwk, encryption_key)?);
        }
        info!("No active key found, generating initial key pair");
        let jwk = Self::generate_key_pair(pool, encryption_key).await?;
        Ok(Self::decrypt_jwk(jwk, encryption_key)?)
    }

    /// 根据 kid 查找密钥（解密后返回）
    pub async fn get_key_by_kid(pool: &PgPool, kid: &str, encryption_key: &str) -> Result<Option<Jwk>> {
        match JwkRepo::find_by_kid(pool, kid).await? {
            Some(jwk) => Ok(Some(Self::decrypt_jwk(jwk, encryption_key)?)),
            None => Ok(None),
        }
    }

    /// 轮换密钥：生成新密钥并激活，旧密钥保留用于验证
    pub async fn rotate_key(pool: &PgPool, encryption_key: &str) -> Result<Jwk> {
        let new_key = Self::generate_key_pair(pool, encryption_key).await?;
        JwkRepo::activate(pool, new_key.id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to activate new key: {}", e))?;

        info!("Key rotated: new kid={}", new_key.kid);
        Ok(new_key)
    }

    /// 获取所有密钥（用于 JWKS 端点）
    pub async fn get_jwks(pool: &PgPool) -> Result<Vec<Jwk>> {
        Ok(JwkRepo::find_all(pool).await?)
    }

    /// 哈希授权码（SHA-256 → base64url）
    pub fn hash_token(token: &str) -> String {
        let hash = Sha256::digest(token.as_bytes());
        URL_SAFE_NO_PAD.encode(&hash)
    }

    /// PKCE S256 验证：BASE64URL(SHA256(code_verifier)) == code_challenge
    pub fn verify_pkce_s256(code_verifier: &str, code_challenge: &str) -> bool {
        let hash = Sha256::digest(code_verifier.as_bytes());
        let computed = URL_SAFE_NO_PAD.encode(&hash);
        computed == code_challenge
    }

    /// 构造公钥 JWK JSON
    fn build_public_jwk(signing_key: &SigningKey, kid: &str) -> serde_json::Value {
        let verifying_key = signing_key.verifying_key();
        let point = verifying_key.to_encoded_point(false);

        let x_bytes = point.x().expect("x coordinate");
        let y_bytes = point.y().expect("y coordinate");

        json!({
            "kty": "EC",
            "use": "sig",
            "alg": "ES256",
            "kid": kid,
            "crv": "P-256",
            "x": URL_SAFE_NO_PAD.encode(x_bytes),
            "y": URL_SAFE_NO_PAD.encode(y_bytes),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_deterministic() {
        let token = "my-secret-code";
        let hash1 = KeyService::hash_token(token);
        let hash2 = KeyService::hash_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = KeyService::hash_token("token-a");
        let hash2 = KeyService::hash_token("token-b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_pkce_s256() {
        // 手动计算: SHA256("my-verifier") → base64url
        let verifier = "my-verifier";
        let hash = Sha256::digest(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(&hash);

        assert!(KeyService::verify_pkce_s256(verifier, &challenge));
        assert!(!KeyService::verify_pkce_s256("wrong-verifier", &challenge));
    }

    #[test]
    fn test_build_public_jwk_structure() {
        let signing_key = SigningKey::random(&mut OsRng);
        let jwk = KeyService::build_public_jwk(&signing_key, "test-kid");

        assert_eq!(jwk["kty"], "EC");
        assert_eq!(jwk["use"], "sig");
        assert_eq!(jwk["alg"], "ES256");
        assert_eq!(jwk["kid"], "test-kid");
        assert_eq!(jwk["crv"], "P-256");
        assert!(jwk["x"].is_string());
        assert!(jwk["y"].is_string());
    }
}

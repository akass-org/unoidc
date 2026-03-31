// 通用加密工具函数
//
// 提供通用的哈希和验证功能

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

// HMAC-SHA256 类型别名
pub type HmacSha256 = Hmac<Sha256>;

/// 使用 SHA-256 哈希 token，返回 base64url 编码的结果
///
/// 用于授权码、refresh token 等的存储哈希。
/// 注意：这些是高熵随机值（非用户选择），SHA-256 已足够安全。
/// 如需额外防护，可通过环境变量配置 HMAC 密钥。
pub fn hash_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

/// PKCE S256 验证
///
/// 验证: BASE64URL(SHA256(code_verifier)) == code_challenge
pub fn verify_pkce_s256(code_verifier: &str, code_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let computed_bytes = URL_SAFE_NO_PAD.encode(hash);

    let a = computed_bytes.as_bytes();
    let b = code_challenge.as_bytes();

    if a.len() != b.len() {
        let _ = Sha256::digest(b);
        return false;
    }

    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// 签名会话 ID（用于验证 cookie 真实性）
///
/// 使用 session_secret 作为 HMAC-SHA256 密钥
/// 返回 Base64 URL-safe 编码的签名
pub fn sign_session(session_id: &str, session_secret: &str) -> anyhow::Result<String> {
    let mut mac = HmacSha256::new_from_slice(session_secret.as_bytes())
        .map_err(|e| anyhow::anyhow!("Invalid session secret: {}", e))?;
    mac.update(session_id.as_bytes());
    let result = mac.finalize();
    let signature = URL_SAFE_NO_PAD.encode(result.into_bytes());
    Ok(signature)
}

/// 验证会话签名
///
/// 验证 cookie 中的签名是否有效
pub fn verify_session_signature(session_id: &str, signature: &str, session_secret: &str) -> bool {
    let mut mac = match HmacSha256::new_from_slice(session_secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(session_id.as_bytes());

    let expected = match URL_SAFE_NO_PAD.decode(signature) {
        Ok(b) => b,
        Err(_) => return false,
    };

    mac.verify_slice(&expected).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_deterministic() {
        let token = "my-secret-code";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = hash_token("token-a");
        let hash2 = hash_token("token-b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_pkce_s256() {
        let verifier = "my-verifier";
        let hash = Sha256::digest(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        assert!(verify_pkce_s256(verifier, &challenge));
        assert!(!verify_pkce_s256("wrong-verifier", &challenge));
    }

    #[test]
    fn test_session_signature() {
        let session_id = "test-session-id-123";
        let secret = "my-super-secret-key-for-testing-32chars";

        // 签名会话
        let signature = sign_session(session_id, secret).unwrap();
        assert!(!signature.is_empty());

        // 验证签名
        assert!(verify_session_signature(session_id, &signature, secret));

        // 错误的签名应该失败
        assert!(!verify_session_signature(session_id, "invalid-sig", secret));
        assert!(!verify_session_signature("wrong-session", &signature, secret));
        assert!(!verify_session_signature(session_id, &signature, "wrong-secret"));
    }
}

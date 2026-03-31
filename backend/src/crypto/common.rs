// 通用加密工具函数
//
// 提供通用的哈希和验证功能

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

/// 使用 SHA-256 哈希 token，返回 base64url 编码的结果
///
/// 用于授权码、refresh token 等的存储哈希
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
}

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{thread_rng, RngCore};

/// 生成指定长度的安全随机令牌
///
/// 使用操作系统提供的密码学安全随机数生成器（CSPRNG）
/// 返回 Base64 URL 安全编码的字符串（无填充）
pub fn generate_secure_token(byte_length: usize) -> Result<String> {
    let mut bytes = vec![0u8; byte_length];
    thread_rng()
        .try_fill_bytes(&mut bytes)
        .map_err(|e| anyhow::anyhow!("Failed to generate random bytes: {}", e))?;

    Ok(URL_SAFE_NO_PAD.encode(&bytes))
}

/// 生成 OAuth 2.0 授权码
///
/// 授权码应该是短期有效的、一次性使用的
/// 使用 32 字节（256 位）的随机值
pub fn generate_authorization_code() -> Result<String> {
    generate_secure_token(32)
}

/// 生成刷新令牌
///
/// 刷新令牌应该是长期有效的、可撤销的
/// 使用 64 字节（512 位）的随机值，提供更高的安全性
pub fn generate_refresh_token() -> Result<String> {
    generate_secure_token(64)
}

/// 生成浏览器会话 ID
///
/// 会话 ID 用于关联用户的浏览器会话
/// 使用 32 字节（256 位）的随机值
pub fn generate_session_id() -> Result<String> {
    generate_secure_token(32)
}

/// 生成 PKCE code_verifier
///
/// 根据 RFC 7636，code_verifier 必须满足：
/// - 长度在 43-128 字符之间
/// - 使用 unreserved 字符：A-Z, a-z, 0-9, -, ., _, ~
/// - 具有足够的熵（至少 256 位）
///
/// 我们使用 96 字节，Base64 编码后为 128 字符
pub fn generate_pkce_code_verifier() -> Result<String> {
    generate_secure_token(96)
}

/// 生成 CSRF Token
///
/// CSRF token 用于防止跨站请求伪造攻击
/// 使用 32 字节（256 位）的随机值
pub fn generate_csrf_token() -> Result<String> {
    generate_secure_token(32)
}

/// 生成客户端 ID
///
/// 客户端 ID 是公开的标识符
/// 使用 16 字节（128 位）的随机值
pub fn generate_client_id() -> Result<String> {
    generate_secure_token(16)
}

/// 生成客户端密钥
///
/// 客户端密钥应该足够长且随机
/// 使用 32 字节（256 位）的随机值
pub fn generate_client_secret() -> Result<String> {
    generate_secure_token(32)
}

/// 生成用户 ID（用于 URL 等）
///
/// 使用 16 字节（128 位）的随机值
pub fn generate_user_id() -> Result<String> {
    generate_secure_token(16)
}

/// 生成组 ID
///
/// 使用 16 字节（128 位）的随机值
pub fn generate_group_id() -> Result<String> {
    generate_secure_token(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token(32).unwrap();
        assert_eq!(token.len(), 43); // 32 bytes -> 43 chars in base64 (no padding)
    }

    #[test]
    fn test_token_uniqueness() {
        let token1 = generate_secure_token(32).unwrap();
        let token2 = generate_secure_token(32).unwrap();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_authorization_code() {
        let code = generate_authorization_code().unwrap();
        assert!(code.len() >= 43);
    }

    #[test]
    fn test_generate_pkce_code_verifier() {
        let verifier = generate_pkce_code_verifier().unwrap();
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
    }
}

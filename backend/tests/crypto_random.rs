// 随机令牌生成测试
use backend::crypto::random;

#[test]
fn generate_secure_token_length() {
    // 32 字节 -> Base64 URL 编码后约 43 字符
    let token = random::generate_secure_token(32).expect("Failed to generate token");
    assert_eq!(token.len(), 43);
}

#[test]
fn generate_secure_token_uniqueness() {
    let token1 = random::generate_secure_token(64).expect("Failed to generate token");
    let token2 = random::generate_secure_token(64).expect("Failed to generate token");

    // 两个令牌应该不同
    assert_ne!(token1, token2);
}

#[test]
fn generate_secure_token_url_safe() {
    let token = random::generate_secure_token(32).expect("Failed to generate token");

    // 令牌应该是 URL 安全的（Base64 URL 编码）
    // 只包含字母、数字、连字符和下划线
    assert!(token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn generate_authorization_code() {
    let code = random::generate_authorization_code().expect("Failed to generate code");

    // 授权码应该足够长（至少 32 字节）
    assert!(code.len() >= 32);

    // 应该是 URL 安全的
    assert!(code
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn generate_refresh_token() {
    let token = random::generate_refresh_token().expect("Failed to generate token");

    // 刷新令牌应该足够长（至少 64 字节）
    assert!(token.len() >= 64);

    // 应该是 URL 安全的
    assert!(token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn generate_session_id() {
    let session_id = random::generate_session_id().expect("Failed to generate session ID");

    // Session ID 应该足够长
    assert!(session_id.len() >= 32);

    // 应该是 URL 安全的
    assert!(session_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn generate_pkce_code_verifier() {
    let verifier = random::generate_pkce_code_verifier().expect("Failed to generate verifier");

    // PKCE code verifier 应该是 43-128 字符
    assert!(verifier.len() >= 43 && verifier.len() <= 128);

    // 应该是 URL 安全的
    assert!(verifier.chars().all(|c| c.is_alphanumeric()
        || c == '-'
        || c == '_'
        || c == '~'
        || c == '.'));
}

#[test]
fn generate_csrf_token() {
    let token = random::generate_csrf_token().expect("Failed to generate CSRF token");

    // CSRF token 应该足够长
    assert!(token.len() >= 32);

    // 应该是 URL 安全的
    assert!(token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

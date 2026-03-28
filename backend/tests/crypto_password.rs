// 密码哈希和验证测试
use backend::crypto::password;

#[test]
fn password_hash_and_verify_success() {
    let password = "test_password_123";
    let hash = password::hash_password(password).expect("Failed to hash password");

    // 哈希应该不为空且不等于原文
    assert!(!hash.is_empty());
    assert_ne!(hash, password);

    // 验证正确的密码
    assert!(password::verify_password(password, &hash).expect("Failed to verify password"));

    // 验证错误的密码应该失败
    assert!(!password::verify_password("wrong_password", &hash).expect("Failed to verify password"));
}

#[test]
fn password_hash_creates_unique_hashes() {
    let password = "same_password";
    let hash1 = password::hash_password(password).expect("Failed to hash password");
    let hash2 = password::hash_password(password).expect("Failed to hash password");

    // 相同的密码应该产生不同的哈希（因为 salt 不同）
    assert_ne!(hash1, hash2);

    // 但两个哈希都应该能验证原密码
    assert!(password::verify_password(password, &hash1).expect("Failed to verify"));
    assert!(password::verify_password(password, &hash2).expect("Failed to verify"));
}

#[test]
fn client_secret_hash_and_verify() {
    let secret = "my_super_secret_client_secret";
    let hash = password::hash_client_secret(secret).expect("Failed to hash client secret");

    // 哈希应该不为空且不等于原文
    assert!(!hash.is_empty());
    assert_ne!(hash, secret);

    // 验证正确的密钥
    assert!(password::verify_client_secret(secret, &hash).expect("Failed to verify client secret"));

    // 验证错误的密钥应该失败
    assert!(!password::verify_client_secret("wrong_secret", &hash).expect("Failed to verify client secret"));
}

#[test]
fn empty_password_handling() {
    // 空密码应该也能被哈希（虽然实际应用中应该在前端验证）
    let hash = password::hash_password("").expect("Failed to hash empty password");
    assert!(password::verify_password("", &hash).expect("Failed to verify empty password"));
}

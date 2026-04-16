use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

/// 密码哈希配置
const PASSWORD_MEMORY_COST: u32 = 19_456; // 19 MiB
const PASSWORD_TIME_COST: u32 = 2; // 2 iterations
const PASSWORD_PARALLELISM: u32 = 1; // 1 thread

/// 客户端密钥哈希配置（更高强度）
const CLIENT_SECRET_MEMORY_COST: u32 = 64_000; // 64 MiB
const CLIENT_SECRET_TIME_COST: u32 = 3; // 3 iterations
const CLIENT_SECRET_PARALLELISM: u32 = 4; // 4 threads

/// 使用 Argon2id 对密码进行哈希
///
/// Argon2id 是当前推荐的密码哈希算法，具有抗 GPU 破解能力
/// 使用随机 salt，每个密码的哈希都不同
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    let params = Params::new(
        PASSWORD_MEMORY_COST,
        PASSWORD_TIME_COST,
        PASSWORD_PARALLELISM,
        None,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create Argon2id parameters: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    Ok(hash)
}

/// 验证密码是否匹配哈希
///
/// 返回 Ok(true) 表示匹配，Ok(false) 表示不匹配
///
/// 注意：从 PHC 格式的哈希字符串中自动提取参数，无需硬编码
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

    // 使用 Argon2 默认实例，它会从 parsed_hash 中自动提取参数
    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Failed to verify password: {}", e)),
    }
}

/// 使用更高强度参数对客户端密钥进行哈希
///
/// 客户端密钥需要更强的保护，使用更高的内存和时间成本
pub fn hash_client_secret(secret: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    let params = Params::new(
        CLIENT_SECRET_MEMORY_COST,
        CLIENT_SECRET_TIME_COST,
        CLIENT_SECRET_PARALLELISM,
        None,
    )
    .map_err(|e| {
        anyhow::anyhow!(
            "Failed to create Argon2id parameters for client secret: {}",
            e
        )
    })?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let hash = argon2
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash client secret: {}", e))?
        .to_string();

    Ok(hash)
}

/// 验证客户端密钥是否匹配哈希
///
/// 返回 Ok(true) 表示匹配，Ok(false) 表示不匹配
///
/// 注意：从 PHC 格式的哈希字符串中自动提取参数，无需硬编码
pub fn verify_client_secret(secret: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse client secret hash: {}", e))?;

    // 使用 Argon2 默认实例，它会从 parsed_hash 中自动提取参数
    let argon2 = Argon2::default();

    match argon2.verify_password(secret.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Failed to verify client secret: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_client_secret_hash_and_verify() {
        let secret = "my_client_secret";
        let hash = hash_client_secret(secret).unwrap();

        assert!(verify_client_secret(secret, &hash).unwrap());
        assert!(!verify_client_secret("wrong", &hash).unwrap());
    }
}

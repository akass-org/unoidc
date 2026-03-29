// 私钥加密模块
//
// 使用 AES-256-GCM 对数据库中存储的签名私钥进行加密

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

/// 使用 HKDF-SHA256 从配置的加密密钥字符串派生 32 字节 AES 密钥
fn derive_key(encryption_key: &str, salt: &[u8; 32]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), encryption_key.as_bytes());
    let mut key = [0u8; 32];
    hkdf.expand(b"aes-256-gcm key", &mut key)
        .expect("HKDF expand should not fail");
    key
}

/// 加密私钥 PEM
pub fn encrypt_private_key(pem: &str, encryption_key: &str) -> anyhow::Result<String> {
    // 生成随机 salt (32 字节)
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);

    // 使用 HKDF 派生密钥
    let key = Aes256Gcm::new_from_slice(&derive_key(encryption_key, &salt))?;

    // 生成随机 nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // 加密
    let ciphertext = key
        .encrypt(&nonce, pem.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // 将 salt + nonce + ciphertext 拼接，一起 base64 编码
    // 格式: [32 bytes salt][12 bytes nonce][ciphertext]
    let mut combined = Vec::with_capacity(32 + 12 + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&combined))
}

/// 解密私钥 PEM
pub fn decrypt_private_key(encrypted: &str, encryption_key: &str) -> anyhow::Result<String> {
    let combined = STANDARD
        .decode(encrypted)
        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;

    // 最小长度: 32 (salt) + 12 (nonce) + 1 (最少密文)
    if combined.len() < 45 {
        return Err(anyhow::anyhow!("Encrypted data too short"));
    }

    // 解析 salt + nonce + ciphertext
    let (salt_bytes, rest) = combined.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(12);

    let salt: [u8; 32] = salt_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid salt length"))?;
    let nonce_arr: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
    let nonce = Nonce::from(nonce_arr);

    // 使用 HKDF 派生密钥
    let key = Aes256Gcm::new_from_slice(&derive_key(encryption_key, &salt))?;

    // 解密
    let plaintext = key
        .decrypt(&nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let pem = "-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgtest\n-----END PRIVATE KEY-----";
        let key = "test-encryption-key-32-characters!!";

        let encrypted = encrypt_private_key(pem, key).unwrap();
        let decrypted = decrypt_private_key(&encrypted, key).unwrap();

        assert_eq!(pem, decrypted);
    }

    #[test]
    fn test_different_salt_each_time() {
        let pem = "test-pem-content";
        let key = "test-key";

        let encrypted1 = encrypt_private_key(pem, key).unwrap();
        let encrypted2 = encrypt_private_key(pem, key).unwrap();

        // 相同输入应产生不同密文（因为 salt 和 nonce 都随机）
        assert_ne!(encrypted1, encrypted2);

        // 但都能正确解密
        assert_eq!(pem, decrypt_private_key(&encrypted1, key).unwrap());
        assert_eq!(pem, decrypt_private_key(&encrypted2, key).unwrap());
    }

    #[test]
    fn test_wrong_key_fails() {
        let pem = "test-pem-content";
        let key1 = "correct-encryption-key-32-chars!";
        let key2 = "wrong-encryption-key-32-chars!!";

        let encrypted = encrypt_private_key(pem, key1).unwrap();
        assert!(decrypt_private_key(&encrypted, key2).is_err());
    }

    #[test]
    fn test_encrypted_format_has_salt() {
        let pem = "test-pem-content";
        let key = "test-encryption-key";

        let encrypted = encrypt_private_key(pem, key).unwrap();
        let combined = STANDARD.decode(&encrypted).unwrap();

        // 新格式应该是: 32 (salt) + 12 (nonce) + ciphertext
        assert!(combined.len() >= 45);
    }
}

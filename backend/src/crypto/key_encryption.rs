// 私钥加密模块
//
// 使用 AES-256-GCM 对数据库中存储的签名私钥进行加密

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};

/// 从配置的加密密钥字符串派生 32 字节 AES 密钥
fn derive_key(encryption_key: &str) -> [u8; 32] {
    let hash = Sha256::digest(encryption_key.as_bytes());
    hash.into()
}

/// 加密私钥 PEM
pub fn encrypt_private_key(pem: &str, encryption_key: &str) -> anyhow::Result<String> {
    let key = Aes256Gcm::new_from_slice(&derive_key(encryption_key))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = key
        .encrypt(&nonce, pem.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
    // 将 nonce 拼接到密文前面，一起 base64 编码
    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&combined))
}

/// 解密私钥 PEM
pub fn decrypt_private_key(encrypted: &str, encryption_key: &str) -> anyhow::Result<String> {
    let key = Aes256Gcm::new_from_slice(&derive_key(encryption_key))?;
    let combined = STANDARD
        .decode(encrypted)
        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;
    if combined.len() < 12 {
        return Err(anyhow::anyhow!("Encrypted data too short"));
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce_arr: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
    let nonce = Nonce::from(nonce_arr);
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
    fn test_different_nonce_each_time() {
        let pem = "test-pem-content";
        let key = "test-key";

        let encrypted1 = encrypt_private_key(pem, key).unwrap();
        let encrypted2 = encrypt_private_key(pem, key).unwrap();

        // 相同输入应产生不同密文（因为 nonce 随机）
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
}

//! API Key 加密模块
//!
//! 实现 API Key 的 AES-256-CBC 加密和解密

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::Result;
use rand::Rng;
use sha2::{Digest, Sha256};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// 加密配置
const ENCRYPTION_SALT: &str = "droid-account-salt";

/// 从密码派生加密密钥
fn derive_key(password: &str) -> [u8; 32] {
    // 使用 SHA256 作为简单的密钥派生（生产环境应使用 scrypt 或 argon2）
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(ENCRYPTION_SALT.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// 加密敏感数据
pub fn encrypt_sensitive_data(plaintext: &str, encryption_key: &str) -> Result<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let key = derive_key(encryption_key);
    let iv: [u8; 16] = rand::thread_rng().gen();

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());

    let plaintext_bytes = plaintext.as_bytes();
    let mut buffer = vec![0u8; plaintext_bytes.len() + 16]; // 预留 padding 空间
    buffer[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext_bytes.len())
        .map_err(|e| anyhow::anyhow!("加密失败: {:?}", e))?;

    // 格式: iv_hex:ciphertext_hex
    Ok(format!("{}:{}", hex::encode(iv), hex::encode(ciphertext)))
}

/// 解密敏感数据
pub fn decrypt_sensitive_data(encrypted_text: &str, encryption_key: &str) -> Result<String> {
    if encrypted_text.is_empty() {
        return Ok(String::new());
    }

    let parts: Vec<&str> = encrypted_text.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("加密数据格式无效");
    }

    let iv = hex::decode(parts[0]).map_err(|e| anyhow::anyhow!("IV 解码失败: {}", e))?;
    let ciphertext =
        hex::decode(parts[1]).map_err(|e| anyhow::anyhow!("密文解码失败: {}", e))?;

    if iv.len() != 16 {
        anyhow::bail!("IV 长度无效");
    }

    let key = derive_key(encryption_key);
    let mut iv_array = [0u8; 16];
    iv_array.copy_from_slice(&iv);

    let cipher = Aes256CbcDec::new(&key.into(), &iv_array.into());

    let mut buffer = ciphertext.clone();
    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| anyhow::anyhow!("解密失败: {:?}", e))?;

    String::from_utf8(plaintext.to_vec()).map_err(|e| anyhow::anyhow!("UTF-8 解码失败: {}", e))
}

/// 计算 API Key 哈希（用于去重）
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "test-api-key-12345";
        let key = "test-encryption-key";

        let encrypted = encrypt_sensitive_data(plaintext, key).unwrap();
        assert!(!encrypted.is_empty());
        assert!(encrypted.contains(':'));

        let decrypted = decrypt_sensitive_data(&encrypted, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_hash_api_key() {
        let api_key = "test-api-key";
        let hash = hash_api_key(api_key);
        assert_eq!(hash.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_empty_string() {
        let key = "test-key";
        let encrypted = encrypt_sensitive_data("", key).unwrap();
        assert!(encrypted.is_empty());

        let decrypted = decrypt_sensitive_data("", key).unwrap();
        assert!(decrypted.is_empty());
    }
}

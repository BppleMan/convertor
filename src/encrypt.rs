use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, Key, Nonce};
use color_eyre::eyre::eyre;
use color_eyre::Result;

pub fn encrypt(secret: &[u8], plaintext: &str) -> Result<String> {
    let secret = normalize_key(secret);
    let key = Key::from_slice(&secret);
    let cipher = ChaCha20Poly1305::new(key);

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    // 加密数据
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| eyre!(e))?;

    // 返回 base64 编码的 nonce 和密文
    let result =
        format!("{}:{}", STANDARD.encode(nonce), STANDARD.encode(ciphertext));
    Ok(result)
}

/// 解密函数
pub fn decrypt(secret: &[u8], encrypted: &str) -> Result<String> {
    // 初始化加密密钥
    let secret = normalize_key(secret);
    let key = Key::from_slice(&secret);
    let cipher = ChaCha20Poly1305::new(key);

    // 分离 base64 编码的 nonce 和密文
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 2 {
        return Err(eyre!("Invalid encrypted data format"));
    }

    let nonce = STANDARD.decode(parts[0])?;
    let ciphertext = STANDARD.decode(parts[1])?;

    // 解密数据
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|e| eyre!(e))?;
    Ok(String::from_utf8(plaintext)?)
}

fn normalize_key(key: &[u8]) -> [u8; 32] {
    let mut normalized = [0u8; 32];
    let len = key.len().min(32);
    normalized[..len].copy_from_slice(&key[..len]);
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;

    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        color_eyre::install()?;

        let secret = b"abcdefg"; // 密钥必须是32字节
        let message = "This is a secret message.";

        // 加密
        let encrypted = encrypt(secret, message)?;
        println!("Encrypted: {}", encrypted);

        // 解密
        let decrypted = decrypt(secret, &encrypted)?;
        println!("Decrypted: {}", decrypted);

        assert_eq!(message, decrypted);

        Ok(())
    }
}

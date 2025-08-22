use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptError {
    #[error("无法分离 nonce 与密文")]
    SplitError,

    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("加密失败: {0}")]
    Encrypt(chacha20poly1305::aead::Error),

    #[error("解码 base64 字符串失败")]
    DecodeError(#[source] base64::DecodeError),
}

pub fn encrypt(secret: &[u8], plaintext: &str) -> Result<String, EncryptError> {
    let secret = normalize_key(secret);
    let key = Key::from_slice(&secret);
    let cipher = ChaCha20Poly1305::new(key);

    println!("Encrypting {plaintext}");
    #[cfg(debug_assertions)]
    let nonce = {
        println!("Using default nonce in debug mode");
        Nonce::default()
    };
    #[cfg(not(debug_assertions))]
    let nonce = {
        use chacha20poly1305::AeadCore;
        use chacha20poly1305::aead::OsRng;
        ChaCha20Poly1305::generate_nonce(&mut OsRng)
    };

    // 加密数据
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(EncryptError::Encrypt)?;

    // 返回 base64 编码的 nonce 和密文
    let result = format!("{}:{}", STANDARD.encode(nonce), STANDARD.encode(ciphertext));
    println!("Encrypted {result}");
    Ok(result)
}

/// 解密函数
pub fn decrypt(secret: &[u8], encrypted: &str) -> Result<String, EncryptError> {
    // 初始化加密密钥
    let secret = normalize_key(secret);
    let key = Key::from_slice(&secret);
    let cipher = ChaCha20Poly1305::new(key);

    // 分离 base64 编码的 nonce 和密文
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 2 {
        return Err(EncryptError::SplitError);
    }

    let nonce = STANDARD.decode(parts[0]).map_err(EncryptError::DecodeError)?;
    let ciphertext = STANDARD.decode(parts[1]).map_err(EncryptError::DecodeError)?;

    // 解密数据
    let plaintext = match cipher.decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref()) {
        Ok(plaintext) => plaintext,
        Err(e) => panic!("{}", e),
    };

    Ok(String::from_utf8(plaintext)?)
}

fn normalize_key(key: &[u8]) -> [u8; 32] {
    let mut normalized = [0u8; 32];
    let len = key.len().min(32);
    normalized[..len].copy_from_slice(&key[..len]);
    normalized
}

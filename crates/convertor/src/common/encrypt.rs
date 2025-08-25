use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand_core::OsRng;
use std::cell::RefCell;
use thiserror::Error;

// ===== 线程局部：给“当前线程”注入可复现的 RNG =====
// 每个测试线程可在第一行设置自己的种子，互不影响，支持并行。
thread_local! {
    static TL_SEEDED_RNG: RefCell<Option<rand_chacha::ChaCha20Rng>> = const { RefCell::new(None) };
}

/// 在当前线程启用“固定种子”的伪随机数源（可复现，适合快照）
pub fn nonce_rng_use_seed(seed: [u8; 32]) {
    use rand_core::SeedableRng;
    TL_SEEDED_RNG.with(|c| *c.borrow_mut() = Some(rand_chacha::ChaCha20Rng::from_seed(seed)));
}

/// 在当前线程恢复为系统 RNG（生产默认行为）
pub fn nonce_rng_use_system() {
    TL_SEEDED_RNG.with(|c| *c.borrow_mut() = None);
}

/// 统一生成 24B nonce：优先线程局部 RNG，缺省回退 OS RNG
fn gen_nonce24() -> Result<[u8; 24], EncryptError> {
    // 1) 先试线程局部的“固定种子” RNG（可复现、并行互不影响）
    if let Some(n) = TL_SEEDED_RNG.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(rng) = opt.as_mut() {
            use rand_core::RngCore; // infallible
            let mut n = [0u8; 24];
            rng.fill_bytes(&mut n);
            Some(n)
        } else {
            None
        }
    }) {
        return Ok(n);
    }

    // 2) 否则使用 OS RNG（rand_core 0.9 里 OsRng 实现 TryRngCore）
    let mut n = [0u8; 24];
    {
        use rand_core::TryRngCore;
        let mut rng = OsRng;
        rng.try_fill_bytes(&mut n)?;
    }
    Ok(n)
}

const NONCE_LEN: usize = 24;
const NONCE_B64URL_LEN: usize = 32; // 24 bytes -> 32 chars (url-safe, no pad)

fn normalize_key(key: &[u8]) -> [u8; 32] {
    let mut normalized = [0u8; 32];
    let len = key.len().min(32);
    normalized[..len].copy_from_slice(&key[..len]);
    normalized
}

pub fn encrypt(secret: &[u8], plaintext: &str) -> Result<String, EncryptError> {
    let norm_key = normalize_key(secret);
    let key = Key::from_slice(&norm_key);
    let cipher = XChaCha20Poly1305::new(key);

    // 统一从线程局部/OsRng 取 nonce
    let nonce_bytes = gen_nonce24()?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| EncryptError::Encrypt)?;

    // URL-safe, no padding；不加任何分隔符
    let mut out = String::with_capacity(NONCE_B64URL_LEN + (ciphertext.len() * 4).div_ceil(3));
    out.push_str(&B64URL.encode(nonce));
    out.push_str(&B64URL.encode(ciphertext));
    Ok(out)
}

pub fn decrypt(secret: &[u8], token: &str) -> Result<String, EncryptError> {
    if token.len() < NONCE_B64URL_LEN {
        return Err(EncryptError::NonceLength);
    }
    let (nonce_part, ct_part) = token.split_at(NONCE_B64URL_LEN);

    // 先解 nonce
    let nonce_raw = B64URL.decode(nonce_part).map_err(EncryptError::DecodeError)?;
    if nonce_raw.len() != NONCE_LEN {
        return Err(EncryptError::NonceLength);
    }
    let nonce = XNonce::from_slice(&nonce_raw);

    // 再解密文
    let ciphertext = B64URL.decode(ct_part).map_err(EncryptError::DecodeError)?;

    let norm_key = normalize_key(secret);
    let key = Key::from_slice(&norm_key);
    let cipher = XChaCha20Poly1305::new(key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| EncryptError::Decrypt)?;

    Ok(String::from_utf8(plaintext)?)
}

#[derive(Debug, Error)]
pub enum EncryptError {
    #[error("无法分离 nonce 与密文")]
    SplitError,

    #[error("nonce 长度不合法")]
    NonceLength,

    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    OsError(#[from] rand_core::OsError),

    #[error("加密失败")]
    Encrypt,

    #[error("解密失败")]
    Decrypt,

    #[error("解码 base64 字符串失败")]
    DecodeError(#[source] base64::DecodeError),
}

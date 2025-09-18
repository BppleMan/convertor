use thiserror::Error;

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

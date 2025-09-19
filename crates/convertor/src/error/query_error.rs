use crate::error::{EncodeUrlError, EncryptError, ParseUrlError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("无法加密/解密 raw_sub_url: {0}")]
    EncryptError(#[from] EncryptError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    Parse(#[from] ParseUrlError),

    #[error(transparent)]
    Encode(#[from] EncodeUrlError),
}

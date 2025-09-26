use crate::error::{EncodeUrlError, EncryptError, ParseUrlError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("查询参数不能为空")]
    EmptyQuery,

    #[error("没有找到 Host 头: {0}")]
    NoHost(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("无法加密/解密 raw_sub_url: {0}")]
    EncryptError(#[from] EncryptError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    Parse(#[from] ParseUrlError),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Encode(#[from] EncodeUrlError),
}

use crate::common::config::proxy_client::ParseClientError;
use crate::common::config::sub_provider::ParseProviderError;
use crate::common::encrypt::EncryptError;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("无法加密/解密 raw_sub_url: {0}")]
    EncryptError(#[from] EncryptError),

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Encode(#[from] EncodeError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("无法从 URL 中解析 ConvertorUrl: 缺少查询参数: {0}")]
    NotFoundParam(&'static str),

    #[error("无法从 URL 中解析 ConvertorUrl: 没有查询字符串")]
    UrlNoQuery(String),

    #[error(transparent)]
    ParseClientError(#[from] ParseClientError),

    #[error(transparent)]
    ParseProviderError(#[from] ParseProviderError),

    #[error(transparent)]
    ParseServerError(#[from] url::ParseError),

    #[error(transparent)]
    ParseNumError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("构造 rule-provider URL 失败: 必须提供 policy")]
    RuleProviderNoPolicy,

    #[error("无法提取 raw_sub_url 的主机和端口信息: {0}")]
    NoRawSubHost(String),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

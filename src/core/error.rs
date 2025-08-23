use crate::core::profile::rule::Rule;
use crate::core::url_builder::UrlBuilderError;
use crate::server::query::QueryError;
use thiserror::Error;

/// 所有解析失败场景的统一错误
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("无法从 UrlBuilder 中获取 sub_host")]
    SubHost,

    #[error("缺少必要的原始配置")]
    MissingRawProfile,

    #[error("缺少密钥")]
    MissingSecret,

    #[error("规则解析失败 (第 {line} 行): {reason}")]
    Rule { line: usize, reason: String },

    #[error("规则类型解析失败 (第 {line} 行): {reason}")]
    RuleType { line: usize, reason: String },

    #[error("代理解析失败 (第 {line} 行): {reason}")]
    Proxy { line: usize, reason: String },

    #[error("代理组解析失败 (第 {line} 行): {reason}")]
    ProxyGroup { line: usize, reason: String },

    #[error("缺少必要配置段: {0}")]
    SectionMissing(&'static str),

    #[error("无法将: {0} 转换为 ProviderRule")]
    IntoProviderRule(Rule),

    #[error(transparent)]
    QueryError(#[from] QueryError),

    #[error(transparent)]
    ConvertorUrlError(#[from] UrlBuilderError),

    #[error(transparent)]
    RenderError(#[from] RenderError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("渲染失败: {0}")]
    Render(String),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}

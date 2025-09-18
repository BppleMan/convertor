use crate::provider_api::ApiFailed;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderApiError {
    #[error("构建 {name} 失败: {source}")]
    BuildRequestError {
        name: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("{name} 请求失败: {source}")]
    ExecutorError {
        name: String,
        #[source]
        source: Box<ProviderApiError>,
    },

    #[error("从 {name} 请求读取 text 失败: {source}")]
    ResponseError {
        name: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("解析 {name}[{path}] 失败: {source}")]
    JsonPathError {
        name: String,
        path: String,
        #[source]
        source: jsonpath_lib::JsonPathError,
    },

    #[error("未找到 {name}[{path}]")]
    JsonPathNotFound { name: String, path: String },

    #[error("API [{name}] 响应失败: {source}")]
    ApiFailed {
        name: String,
        #[source]
        source: Box<ApiFailed>,
    },

    #[error(transparent)]
    UrlError(#[from] url::ParseError),

    #[error(transparent)]
    InnerError(#[from] Arc<ProviderApiError>),

    #[error("{0}")]
    Other(String),
}

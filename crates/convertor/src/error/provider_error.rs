use crate::config::subscription_config::Headers;
use headers::HeaderMap;
use reqwest::{Method, StatusCode};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ProviderError {
    // #[error("{name} 请求失败: {source}")]
    // ExecutorError {
    //     name: String,
    //     #[source]
    //     source: Box<ProviderApiError>,
    // },
    #[error("请求失败, {reason}: {source}")]
    RequestError {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
    },

    // #[error("解析 {name}[{path}] 失败: {source}")]
    // JsonPathError {
    //     name: String,
    //     path: String,
    //     #[source]
    //     source: jsonpath_lib::JsonPathError,
    // },

    // #[error("未找到 {name}[{path}]")]
    // JsonPathNotFound { name: String, path: String },
    #[error(transparent)]
    ApiFailed(#[from] ApiFailed),

    #[error(transparent)]
    UrlError(#[from] url::ParseError),

    // #[error(transparent)]
    // InnerError(#[from] Arc<ProviderApiError>),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub struct ApiFailed {
    pub request_url: Url,
    pub request_method: Method,
    pub request_headers: Headers,
    pub response_status: StatusCode,
    pub response_headers: Headers,
    pub response_body: String,
}

impl Display for ApiFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] {}", self.request_method, self.request_url)?;
        writeln!(f, "Request Headers:")?;
        for (key, value) in self.request_headers.iter() {
            writeln!(f, "\t{key}: {value:?}")?;
        }
        writeln!(f, "Response Status: {}", self.response_status)?;
        writeln!(f, "Response Headers:")?;
        for (key, value) in self.response_headers.iter() {
            writeln!(f, "\t{key}: {value:?}")?;
        }
        writeln!(f, "Response Body:")?;
        write!(f, "{}", self.response_body)
    }
}

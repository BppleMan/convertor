use crate::config::subscription_config::Headers;
use reqwest::Version;
use reqwest::{Method, StatusCode};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("{reason}: {source}\n{request_info}")]
    RequestError {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        request_info: RequestInfo,
    },

    #[error("{reason}: {source}\n{response_info}")]
    ResponseError {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        response_info: ResponseInfo,
    },

    #[error(transparent)]
    ApiFailed(#[from] Box<ApiFailed>),

    #[error(transparent)]
    UrlError(#[from] url::ParseError),

    #[error(transparent)]
    InnerError(#[from] Arc<ProviderError>),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub struct ApiFailed {
    pub request: RequestInfo,
    pub response: ResponseInfo,
}

#[derive(Debug, Error, Clone)]
pub struct RequestInfo {
    pub req_id: String,
    pub url: Url,
    pub method: Method,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub headers: Option<Headers>,
    pub body: Option<String>,
}

#[derive(Debug, Error, Clone)]
pub struct ResponseInfo {
    pub final_url: Url,
    pub status: StatusCode,
    pub status_text: Option<String>,
    pub headers: Headers,
    pub version: Version,
    pub body: Option<String>,
}

impl Display for ApiFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Request:")?;
        write!(f, "{}", self.request)?;
        writeln!(f, "Response:")?;
        write!(f, "{}", self.response)
    }
}

impl RequestInfo {
    pub fn new(url: Url, method: Method) -> Self {
        Self {
            req_id: uuid::Uuid::new_v4().to_string(),
            url,
            method,
            scheme: None,
            host: None,
            port: None,
            path: None,
            query: None,
            headers: None,
            body: None,
        }
    }

    pub fn patch(mut self, request: &reqwest::Request) -> Self {
        // Request 侧字段
        self.url = request.url().clone();
        self.method = request.method().clone();
        self.scheme = Some(request.url().scheme().to_string());
        self.host = request.url().host_str().map(|s| s.to_string());
        self.port = request.url().port();
        self.path = Some(request.url().path().to_string());
        self.query = request.url().query().map(|s| s.to_string());
        self.headers = Some(Headers::from_header_map(request.headers().clone()));
        if let Some(body) = request.body().and_then(|b| b.as_bytes()) {
            self.body = Some(String::from_utf8_lossy(body).to_string());
        }
        self
    }
}

impl Display for RequestInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Request ID: {}", self.req_id)?;
        writeln!(f, "[{}] {}", self.method, self.url)?;
        if let Some(headers) = &self.headers {
            writeln!(f, "Headers:")?;
            for (i, (key, value)) in headers.iter().enumerate() {
                if i == 0 {
                    write!(f, "\t{key}: {value:?}")?;
                } else {
                    write!(f, "\n\t{key}: {value:?}")?;
                }
            }
        }
        if let Some(body) = &self.body {
            writeln!(f, "\nBody:")?;
            write!(f, "{}", body)?;
        }
        Ok(())
    }
}

impl Display for ResponseInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Status: {}", self.status)?;
        writeln!(f, "Headers:")?;
        for (i, (key, value)) in self.headers.iter().enumerate() {
            if i == 0 {
                write!(f, "\t{key}: {value:?}")?;
            } else {
                write!(f, "\n\t{key}: {value:?}")?;
            }
        }
        writeln!(f, "\nVersion: {:?}", self.version)?;
        if let Some(body) = &self.body {
            writeln!(f, "\nBody:")?;
            write!(f, "{}", body)
        } else {
            Ok(())
        }
    }
}

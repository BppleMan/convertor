use axum::http::request::Parts;
use convertor::config::subscription_config::Headers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct RequestSnapshot {
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub uri: String,
    pub headers: Headers,
}

impl RequestSnapshot {
    pub fn from_parts(scheme: String, host: String, parts: Parts) -> Self {
        let method = parts.method.to_string();
        let uri = parts.uri.to_string();
        let headers = Headers::from_header_map(parts.headers);
        Self {
            method,
            scheme,
            host,
            uri,
            headers,
        }
    }
}

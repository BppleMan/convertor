use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequestConfig {
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub cookie: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl RequestConfig {
    pub fn template() -> Self {
        Self {
            user_agent: Some(concat!("Convertor/", env!("CARGO_PKG_VERSION")).to_string()),
            auth_token: Some("optional:boslife.auth_token".to_string()),
            cookie: Some("optional:boslife.cookie".to_string()),
            headers: [
                ("Host", "http://127.0.0.1:8008"),
                ("Accept", "application/json"),
                ("Content-Type", "application/json"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        }
    }

    pub fn patch_request(
        &self,
        request_builder: RequestBuilder,
        auth_token: Option<impl AsRef<str>>,
    ) -> RequestBuilder {
        let mut request_builder = request_builder;
        if let Some(cookie) = &self.cookie {
            request_builder = request_builder.header("Cookie", cookie);
        }
        if let Some(user_agent) = &self.user_agent {
            request_builder = request_builder.header("User-Agent", user_agent);
        }
        for (k, v) in &self.headers {
            request_builder = request_builder.header(k, v);
        }
        match (auth_token, self.auth_token.as_ref()) {
            (Some(auth_token), _) => {
                request_builder = request_builder.header("Authorization", auth_token.as_ref());
            }
            (None, Some(auth_token)) => {
                request_builder = request_builder.header("Authorization", auth_token);
            }
            _ => {}
        };
        request_builder
    }
}

impl Hash for RequestConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_agent.hash(state);
        self.auth_token.hash(state);
        self.cookie.hash(state);
        for (k, v) in &self.headers {
            k.hash(state);
            v.hash(state);
        }
    }
}

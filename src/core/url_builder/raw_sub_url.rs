use crate::common::config::proxy_client::ProxyClient;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSubUrl {
    pub server: Url,
    pub flag: ProxyClient,
}

impl From<&RawSubUrl> for Url {
    fn from(value: &RawSubUrl) -> Self {
        let mut url = value.server.clone();
        url.query_pairs_mut().append_pair("flag", value.flag.as_str());
        url
    }
}

impl Display for RawSubUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}&flag={}", self.server, self.flag)
    }
}

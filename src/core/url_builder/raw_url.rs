use crate::common::config::proxy_client::ProxyClient;
use std::fmt::Display;
use url::Url;

#[derive(Debug, Clone)]
pub struct RawUrl {
    pub server: Url,
    pub flag: ProxyClient,
}

impl From<&RawUrl> for Url {
    fn from(value: &RawUrl) -> Self {
        let mut url = value.server.clone();
        url.query_pairs_mut().append_pair("flag", value.flag.as_str());
        url
    }
}

impl Display for RawUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Url::from(self))
    }
}

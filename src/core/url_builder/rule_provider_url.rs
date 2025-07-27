use crate::server::query::rule_provider_query::RuleProviderQuery;
use std::fmt::Display;
use url::Url;

#[derive(Debug, Clone)]
pub struct RuleProviderUrl {
    pub server: Url,
    pub path: String,
    pub query: RuleProviderQuery,
}

impl From<&RuleProviderUrl> for Url {
    fn from(value: &RuleProviderUrl) -> Self {
        let mut url = value.server.clone();
        url.set_path(&value.path);
        url.set_query(Some(&value.query.encode_to_query_string()));
        url
    }
}

impl Display for RuleProviderUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}?{}",
            self.server.as_str().trim_end_matches('/'),
            self.path.trim_start_matches('/'),
            self.query.encode_to_query_string(),
        )
    }
}

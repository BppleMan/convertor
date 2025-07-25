use crate::core::query::convertor_query::ConvertorQuery;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorUrl {
    pub server: Url,
    pub path: String,
    pub query: ConvertorQuery,
}

impl From<&ConvertorUrl> for Url {
    fn from(value: &ConvertorUrl) -> Self {
        let mut url = value.server.clone();
        url.set_path(&value.path);
        url.set_query(Some(&value.query.encode_to_query_string()));
        url
    }
}

impl Display for ConvertorUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}?{}",
            self.server.as_str().trim_end_matches('/'),
            self.path.trim_start_matches('/'),
            self.query.encode_to_query_string(),
        )
    }
}

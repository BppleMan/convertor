use crate::server::query::sub_logs_query::SubLogsQuery;
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone)]
pub struct SubLogsUrl {
    pub server: Url,
    pub path: String,
    pub query: SubLogsQuery,
}

impl From<&SubLogsUrl> for Url {
    fn from(value: &SubLogsUrl) -> Self {
        let mut url = value.server.clone();
        url.set_path(&value.path);
        url.set_query(Some(&value.query.encode_to_query_string()));
        url
    }
}

impl Display for SubLogsUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}?{}",
            self.server.as_str().trim_end_matches('/'),
            self.path.trim_start_matches('/'),
            self.query.encode_to_query_string()
        )
    }
}

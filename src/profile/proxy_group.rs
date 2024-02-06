use crate::profile::policy::Policy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyGroup {
    Select {
        name: String,
        proxies: Vec<Policy>,
    },
    URLTest {
        name: String,
        proxies: Vec<Policy>,
        url: Option<String>,
        interval: Option<u64>,
    },
}

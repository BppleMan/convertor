use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: ProxyGroupType,
    pub proxies: Vec<String>,
}

impl ProxyGroup {
    pub fn new(
        name: String,
        r#type: ProxyGroupType,
        proxies: Vec<String>,
    ) -> Self {
        Self {
            name,
            r#type,
            proxies,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum ProxyGroupType {
    #[serde(rename = "select")]
    Select,
    #[default]
    #[serde(rename = "url-test")]
    UrlTest,
}

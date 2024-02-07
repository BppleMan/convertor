use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: ProxyGroupType,
    pub proxies: Vec<String>,
}

impl ProxyGroup {
    pub fn new(name: String, ty: ProxyGroupType, proxies: Vec<String>) -> Self {
        Self { name, ty, proxies }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum ProxyGroupType {
    #[serde(rename = "select")]
    Select,
    #[default]
    #[serde(rename = "url-test")]
    URLTest,
}

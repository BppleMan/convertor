use color_eyre::eyre::eyre;
use color_eyre::Report;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Default, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: ProxyGroupType,
    pub proxies: Vec<String>,
    #[serde(skip)]
    pub comment: Option<String>,
}

impl ProxyGroup {
    pub fn new(name: String, r#type: ProxyGroupType, proxies: Vec<String>) -> Self {
        Self {
            name,
            r#type,
            proxies,
            comment: None,
        }
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }
}

#[derive(Debug, Default, Deserialize)]
pub enum ProxyGroupType {
    #[serde(rename = "select")]
    Select,
    #[default]
    #[serde(rename = "url-test")]
    UrlTest,
    Smart,
}

impl ProxyGroupType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyGroupType::Select => "select",
            ProxyGroupType::UrlTest => "url-test",
            ProxyGroupType::Smart => "smart",
        }
    }
}

impl FromStr for ProxyGroupType {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "select" => Ok(ProxyGroupType::Select),
            "url-test" | "test-url" => Ok(ProxyGroupType::UrlTest),
            "smart" => Ok(ProxyGroupType::Smart),
            _ => Err(eyre!("无法识别的策略组类型: {}", s)),
        }
    }
}

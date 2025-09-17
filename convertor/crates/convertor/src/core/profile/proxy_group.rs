use crate::core::error::ParseError;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Default, Debug, Clone, Deserialize)]
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

#[derive(Default, Debug, Clone, Deserialize)]
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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "select" => Ok(ProxyGroupType::Select),
            "url-test" | "test-url" => Ok(ProxyGroupType::UrlTest),
            "smart" => Ok(ProxyGroupType::Smart),
            _ => Err(ParseError::ProxyGroup {
                line: 0,
                reason: format!("无法识别的策略组类型: {}", s),
            }),
        }
    }
}

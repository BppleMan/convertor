use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantArray};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorUrl {
    pub r#type: ConvertorUrlType,
    pub server: Url,
    pub path: Option<String>,
    pub query: Option<String>,
}

impl ConvertorUrl {
    pub fn new(r#type: ConvertorUrlType, server: Url) -> Self {
        Self {
            r#type,
            server,
            path: None,
            query: None,
        }
    }

    pub fn with_path(mut self, path: impl AsRef<str>) -> Self {
        self.path = Some(path.as_ref().to_string());
        self
    }

    pub fn with_query(mut self, query: impl AsRef<str>) -> Self {
        self.query = Some(query.as_ref().to_string());
        self
    }
}

impl From<&ConvertorUrl> for Url {
    fn from(value: &ConvertorUrl) -> Self {
        let mut url = value.server.clone();
        if let Some(path) = &value.path {
            url.set_path(path);
        }
        if let Some(query) = &value.query {
            url.set_query(Some(query));
        }
        url
    }
}

impl Display for ConvertorUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Url::from(self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(Display, IntoStaticStr, AsRefStr, VariantArray, EnumString)]
pub enum ConvertorUrlType {
    Raw,
    RawProfile,
    Profile,
    RuleProvider,
    SubLogs,
}

impl ConvertorUrlType {
    pub fn label(&self) -> &str {
        match self {
            ConvertorUrlType::Raw => "原始订阅链接",
            ConvertorUrlType::RawProfile => "非转换配置订阅链接",
            ConvertorUrlType::Profile => "转换配置订阅链接",
            ConvertorUrlType::RuleProvider => "规则集",
            ConvertorUrlType::SubLogs => "订阅日志链接",
        }
    }
}

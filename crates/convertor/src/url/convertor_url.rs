use crate::core::profile::policy::Policy;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorUrl {
    pub r#type: ConvertorUrlType,
    pub server: Url,
    pub desc: String,
    pub path: Option<String>,
    pub query: Option<String>,
}

impl ConvertorUrl {
    pub fn new(
        r#type: ConvertorUrlType,
        server: Url,
        path: impl Into<String>,
        query: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        let path = Some(path.into());
        let query = Some(query.into());
        let desc = desc.into();

        Self {
            r#type,
            server,
            path,
            query,
            desc,
        }
    }

    pub fn raw(url: Url) -> Self {
        Self {
            r#type: ConvertorUrlType::Raw,
            server: url,
            path: None,
            query: None,
            desc: ConvertorUrlType::Raw.label(),
        }
    }

    pub fn raw_profile(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        Self::new(
            ConvertorUrlType::RawProfile,
            url,
            path,
            query,
            ConvertorUrlType::RawProfile.label(),
        )
    }

    pub fn profile(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        Self::new(
            ConvertorUrlType::Profile,
            url,
            path,
            query,
            ConvertorUrlType::Profile.label(),
        )
    }

    pub fn rule_provider(policy: Policy, url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        let r#type = ConvertorUrlType::RuleProvider(policy);
        let desc = r#type.label();
        Self::new(r#type, url, path, query, desc)
    }

    pub fn sub_logs(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        Self::new(
            ConvertorUrlType::SubLogs,
            url,
            path,
            query,
            ConvertorUrlType::SubLogs.label(),
        )
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

impl From<ConvertorUrl> for Url {
    fn from(value: ConvertorUrl) -> Self {
        Url::from(&value)
    }
}

impl Display for ConvertorUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Url::from(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum ConvertorUrlType {
    Raw,
    RawProfile,
    Profile,
    SubLogs,
    RuleProvider(Policy),
}

impl ConvertorUrlType {
    pub fn label(&self) -> String {
        match self {
            ConvertorUrlType::Raw => "提供商原始订阅".to_string(),
            ConvertorUrlType::RawProfile => "转发原始订阅".to_string(),
            ConvertorUrlType::Profile => "转换器订阅".to_string(),
            ConvertorUrlType::SubLogs => "订阅日志记录".to_string(),
            ConvertorUrlType::RuleProvider(policy) => {
                format!("规则集: {}", SurgeRenderer::render_provider_name_for_policy(policy))
            }
        }
    }
}

impl ConvertorUrlType {
    pub fn variants() -> &'static [Self] {
        &[
            ConvertorUrlType::Raw,
            ConvertorUrlType::RawProfile,
            ConvertorUrlType::Profile,
            ConvertorUrlType::SubLogs,
        ]
    }
}

impl Display for ConvertorUrlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertorUrlType::Raw => write!(f, "raw"),
            ConvertorUrlType::RawProfile => write!(f, "raw_profile"),
            ConvertorUrlType::Profile => write!(f, "profile"),
            ConvertorUrlType::SubLogs => write!(f, "sub_logs"),
            ConvertorUrlType::RuleProvider(_) => write!(f, "rule_provider"),
        }
    }
}

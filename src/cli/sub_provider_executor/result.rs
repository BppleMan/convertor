use color_eyre::owo_colors::OwoColorize;
use std::fmt;
use std::fmt::Display;
use url::Url;

#[derive(Debug, Clone)]
pub struct SubProviderExecutorLink {
    pub label: String,
    pub url: Url,
}

impl SubProviderExecutorLink {
    pub fn raw(url: Url) -> Self {
        Self {
            label: "原始订阅链接:".to_string(),
            url,
        }
    }

    pub fn raw_profile(url: Url) -> Self {
        Self {
            label: "非转换配置订阅链接:".to_string(),
            url,
        }
    }

    pub fn profile(url: Url) -> Self {
        Self {
            label: "转换配置订阅链接:".to_string(),
            url,
        }
    }

    pub fn logs(url: Url) -> Self {
        Self {
            label: "订阅日志链接:".to_string(),
            url,
        }
    }

    pub fn rule_provider(name: String, url: Url) -> Self {
        Self {
            label: format!("规则集: {name}"),
            url,
        }
    }
}

impl Display for SubProviderExecutorLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.label.green().bold())?;
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone)]
pub struct SubProviderExecutorResult {
    pub raw_link: SubProviderExecutorLink,
    pub raw_profile_link: SubProviderExecutorLink,
    pub profile_link: SubProviderExecutorLink,
    pub logs_link: SubProviderExecutorLink,
    pub rule_provider_links: Vec<SubProviderExecutorLink>,
}

impl Display for SubProviderExecutorResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.raw_link)?;
        writeln!(f, "{}", self.profile_link)?;
        writeln!(f, "{}", self.raw_profile_link)?;
        writeln!(f, "{}", self.logs_link)?;
        for link in &self.rule_provider_links {
            writeln!(f, "{link}")?;
        }
        Ok(())
    }
}

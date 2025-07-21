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
    pub fn convertor(url: Url) -> Self {
        Self {
            label: "转换器订阅链接:".to_string(),
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
    pub convertor_link: SubProviderExecutorLink,
    pub logs_link: SubProviderExecutorLink,
    pub policy_links: Vec<SubProviderExecutorLink>,
}

impl Display for SubProviderExecutorResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.raw_link)?;
        writeln!(f, "{}", self.convertor_link)?;
        writeln!(f, "{}", self.logs_link)?;
        for link in &self.policy_links {
            writeln!(f, "{link}")?;
        }
        Ok(())
    }
}

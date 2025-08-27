use crate::core::profile::policy::Policy;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::url::convertor_url::{ConvertorUrl, ConvertorUrlType};
use color_eyre::owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlResult {
    pub raw_url: ConvertorUrl,
    pub raw_profile_url: ConvertorUrl,
    pub profile_url: ConvertorUrl,
    pub sub_logs_url: ConvertorUrl,
    pub rule_provider_urls: Vec<(Policy, ConvertorUrl)>,
}

impl Display for UrlResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.raw_url.r#type.blue())?;
        writeln!(f, "{}", self.raw_url)?;
        writeln!(f, "{}", self.profile_url.r#type.blue())?;
        writeln!(f, "{}", self.profile_url)?;
        writeln!(f, "{}", self.raw_profile_url.r#type.blue())?;
        writeln!(f, "{}", self.raw_profile_url)?;
        writeln!(f, "{}", self.sub_logs_url.r#type.blue())?;
        writeln!(f, "{}", self.sub_logs_url)?;
        writeln!(f, "{}", ConvertorUrlType::RuleProvider.to_string().blue())?;
        for (policy, url) in &self.rule_provider_urls {
            let policy_name = match ClashRenderer::render_provider_name_for_policy(policy) {
                Ok(policy_name) => policy_name,
                Err(e) => {
                    error!("{e:?}");
                    policy.name.clone()
                }
            };
            writeln!(f, "{policy_name}")?;
            writeln!(f, "{url}")?;
        }
        Ok(())
    }
}

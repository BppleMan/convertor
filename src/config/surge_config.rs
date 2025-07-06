use crate::client::Client;
use crate::profile::core::policy::Policy;
use crate::profile::core::rule::Rule;
use crate::profile::renderer::surge_renderer::{
    SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START, SurgeRenderer,
};
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SurgeConfig {
    #[allow(unused)]
    pub surge_dir: PathBuf,
    pub main_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub rules_config_path: PathBuf,
    pub sub_logs_path: PathBuf,
}

impl SurgeConfig {
    pub fn try_new() -> Result<Self> {
        let icloud_env = std::env::var("ICLOUD")?;
        let icloud_path = Path::new(&icloud_env);
        let ns_surge_path = icloud_path
            .parent()
            .ok_or_eyre("not found icloud's parent")?
            .join("iCloud~com~nssurge~inc")
            .join("Documents");
        let main_config_path = ns_surge_path.join("surge").join("surge.conf");
        let default_config_path = ns_surge_path.join("surge").join("BosLife.conf");
        let rules_config_path = ns_surge_path.join("surge").join("rules.dconf");
        let sub_logs_path = ns_surge_path.join("surge").join("subscription_logs.js");
        Ok(Self {
            surge_dir: ns_surge_path,
            main_config_path,
            default_config_path,
            rules_config_path,
            sub_logs_path,
        })
    }

    pub async fn update_surge_config(&self, convertor_url: &UrlBuilder) -> Result<()> {
        // update BosLife.conf subscription
        let managed_config_header =
            Self::build_managed_config_header(convertor_url.build_subscription_url(Client::Surge)?);
        Self::update_conf(&self.default_config_path, &managed_config_header).await?;

        // update surge.conf subscription
        let surge_conf = Self::build_managed_config_header(convertor_url.build_convertor_url(Client::Surge)?);
        Self::update_conf(&self.main_config_path, &surge_conf).await?;

        Ok(())
    }

    pub async fn update_surge_rule_providers(&self, url_builder: &UrlBuilder, policies: &[Policy]) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.rules_config_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();

        let range_of_rule_providers = lines.iter().enumerate().fold(0..=0, |acc, (no, line)| {
            let mut start = *acc.start();
            let mut end = *acc.end();
            if line == SURGE_RULE_PROVIDER_COMMENT_START {
                start = no;
            } else if line == SURGE_RULE_PROVIDER_COMMENT_END {
                end = no;
            }
            start..=end
        });

        let rule_providers = policies
            .iter()
            .map(|policy| {
                let url = url_builder.build_rule_provider_url(Client::Surge, policy)?;
                Rule::surge_rule_provider(policy, url)
            })
            .collect::<Result<Vec<_>>>()?;
        let output = SurgeRenderer::render_rule_providers_with_comment(&rule_providers)?
            .into_iter()
            .map(Cow::Owned)
            .collect::<Vec<_>>();
        lines.splice(range_of_rule_providers, output);
        let content = lines.join("\n");
        tokio::fs::write(&self.rules_config_path, &content).await?;
        Ok(())
    }

    pub async fn update_surge_sub_logs_url(&self, sub_logs_url: impl IntoUrl) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.sub_logs_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(format!(r#"const sub_logs_url = "{}""#, sub_logs_url.as_str()));
        let content = lines.join("\n");
        tokio::fs::write(&self.sub_logs_path, &content).await?;
        Ok(())
    }

    async fn update_conf(config_path: impl AsRef<Path>, sub_url: impl AsRef<str>) -> Result<()> {
        let mut content = tokio::fs::read_to_string(&config_path).await?;
        let mut lines = content.lines().collect::<Vec<_>>();
        lines[0] = sub_url.as_ref();
        content = lines.join("\n");
        tokio::fs::write(&config_path, &content).await?;
        Ok(())
    }

    pub fn build_managed_config_header(url: impl AsRef<str>) -> String {
        format!("#!MANAGED-CONFIG {} interval=259200 strict=true", url.as_ref())
    }
}

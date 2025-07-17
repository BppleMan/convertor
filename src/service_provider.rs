use crate::client::Client;
use crate::convertor_config::ConvertorConfig;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::extract_policies_for_rule_provider;
use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::profile::rule::Rule;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::renderer::surge_renderer::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START};
use crate::core::result::ParseResult;
use crate::service_provider::api::ServiceApi;
use crate::service_provider::args::ServiceProviderArgs;
use crate::url_builder::UrlBuilder;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use color_eyre::owo_colors::OwoColorize;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub mod api;
pub mod args;
pub mod config;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ServiceProvider {
    #[default]
    BosLife,
}

pub struct SubscriptionService {
    pub config: ConvertorConfig,
    pub api: ServiceApi,
}

impl SubscriptionService {
    pub async fn execute(&mut self, args: ServiceProviderArgs) -> Result<()> {
        let client = args.client;
        let url_builder = self.generate_url_builder(&args).await?;
        let raw_profile_content = self.api.get_raw_profile(client).await?;
        let policies = match client {
            Client::Surge => {
                let raw_profile = SurgeProfile::parse(raw_profile_content)?;
                let polices = extract_policies_for_rule_provider(&raw_profile.rules, url_builder.sub_host()?);
                if args.update {
                    self.update_surge_config(&url_builder, &polices).await?;
                }
                polices
            }
            Client::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let polices = extract_policies_for_rule_provider(&raw_profile.rules, url_builder.sub_host()?);
                if args.update {
                    self.update_clash_config(&url_builder, raw_profile).await?;
                }
                polices
            }
        };
        if matches!(client, Client::Clash) && args.update {
            return Ok(());
        }
        println!("{}", "Raw Subscription url:".to_string().green().bold());
        println!("{}", url_builder.build_raw_sub_url(client)?);
        println!("{}", "Convertor url:".to_string().green().bold());
        println!("{}", url_builder.build_convertor_url(client)?);
        println!("{}", "Subscription logs url:".to_string().green().bold());
        println!("{}", url_builder.build_sub_logs_url(&self.config.secret)?);
        for policy in policies {
            match client {
                Client::Surge => println!(
                    "{}",
                    SurgeRenderer::render_provider_name_for_policy(&policy)?.green().bold()
                ),
                Client::Clash => println!(
                    "{}",
                    ClashRenderer::render_provider_name_for_policy(&policy)?.green().bold()
                ),
            }
            println!("{}", url_builder.build_rule_provider_url(client, &policy)?)
        }

        Ok(())
    }

    async fn generate_url_builder(&self, args: &ServiceProviderArgs) -> Result<UrlBuilder> {
        let ServiceProviderArgs {
            client: _client,
            reset,
            update: _update,
            raw_sub_url,
            convertor_url,
            server,
            interval,
            strict,
        } = args;
        let secret = self.config.secret.clone();
        let server = server.clone().unwrap_or_else(|| self.config.server.clone());
        let interval = interval.unwrap_or_else(|| self.config.interval);
        let strict = strict.unwrap_or_else(|| self.config.strict);

        let url_builder = if let Some(convertor_url) = convertor_url {
            UrlBuilder::decode_from_convertor_url(convertor_url.clone(), &self.config.secret)?
        } else if *reset {
            let raw_sub_url = self.api.reset_raw_sub_url().await?;
            UrlBuilder::new(server, secret, raw_sub_url, interval, strict)?
        } else {
            let raw_sub_url = raw_sub_url
                .clone()
                .unwrap_or_else(|| self.config.service_config.raw_sub_url.clone());
            UrlBuilder::new(server, secret, raw_sub_url, interval, strict)?
        };
        Ok(url_builder)
    }

    async fn update_surge_config(&self, url_builder: &UrlBuilder, policies: &[Policy]) -> Result<()> {
        let surge_config = SurgeConfig::try_new()?;
        surge_config.update_surge_config(url_builder).await?;
        surge_config
            .update_surge_sub_logs_url(url_builder.build_sub_logs_url(&self.config.secret)?)
            .await?;
        surge_config.update_surge_rule_providers(url_builder, policies).await?;
        Ok(())
    }

    async fn update_clash_config(&self, url_builder: &UrlBuilder, raw_profile: ClashProfile) -> Result<()> {
        let clash_config = ClashConfig::try_new()?;
        clash_config
            .update_clash_config(url_builder, raw_profile, &self.config.secret)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SurgeConfig {
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

    pub async fn update_surge_config(&self, url_builder: &UrlBuilder) -> Result<()> {
        // update BosLife.conf subscription
        let managed_config_header = Self::build_managed_config_header(
            url_builder.build_raw_sub_url(Client::Surge)?,
            url_builder.interval,
            url_builder.strict,
        );
        Self::update_conf(&self.default_config_path, &managed_config_header).await?;

        // update surge.conf subscription
        let surge_conf = Self::build_managed_config_header(
            url_builder.build_convertor_url(Client::Surge)?,
            url_builder.interval,
            url_builder.strict,
        );
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

        let provider_rules = policies
            .iter()
            .map(|policy| {
                let name = SurgeRenderer::render_provider_name_for_policy(policy)?;
                let url = url_builder.build_rule_provider_url(Client::Surge, policy)?;
                Ok(Rule::surge_rule_provider(policy, name, url))
            })
            .collect::<ParseResult<Vec<_>>>()?;
        let mut output = provider_rules
            .iter()
            .map(SurgeRenderer::render_rule)
            .map(|l| Ok(l.map(Cow::Owned)?))
            .collect::<Result<Vec<_>>>()?;
        output.insert(0, Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_START));
        output.push(Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_END));
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

    pub fn build_managed_config_header(url: impl IntoUrl, interval: u64, strict: bool) -> String {
        format!("#!MANAGED-CONFIG {} interval={interval} strict={strict}", url.as_str())
    }
}

pub struct ClashConfig {
    pub clash_dir: PathBuf,
    pub main_config_path: PathBuf,
}

impl ClashConfig {
    pub fn try_new() -> Result<Self> {
        let home_env = std::env::var("HOME")?;
        let home_path = Path::new(&home_env);
        let clash_dir = home_path.join(".config").join("mihomo");
        let main_config_path = clash_dir.join("config.yaml");
        Ok(Self {
            clash_dir,
            main_config_path,
        })
    }

    pub async fn update_clash_config(
        &self,
        url_builder: &UrlBuilder,
        raw_profile: ClashProfile,
        secret: impl AsRef<str>,
    ) -> Result<()> {
        let mut template = ClashProfile::template()?;
        template.merge(raw_profile, secret)?;
        template.optimize(url_builder)?;
        let clash_config = ClashRenderer::render_profile(&template)?;
        tokio::fs::write(&self.main_config_path, clash_config).await?;
        Ok(())
    }
}

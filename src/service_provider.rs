use crate::client::Client;
use crate::config::convertor_config::ConvertorConfig;
use crate::config::surge_config::SurgeConfig;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::service_provider::subscription_api::boslife_api::BosLifeApi;
use crate::service_provider::subscription_args::SubscriptionArgs;
use crate::url_builder::UrlBuilder;
use color_eyre::owo_colors::OwoColorize;

pub mod subscription_log;
pub mod subscription_api;
pub mod subscription_args;
pub mod subscription_config;

pub struct SubscriptionService {
    pub config: ConvertorConfig,
    pub api: BosLifeApi,
}

impl SubscriptionService {
    pub async fn execute(&mut self, args: SubscriptionArgs) -> color_eyre::Result<()> {
        let client = args.client;
        let url_builder = self.generate_url_builder(&args).await?;
        let raw_sub_url = url_builder.build_subscription_url(client)?;
        let raw_profile_content = self.api.get_raw_profile(raw_sub_url, client).await?;
        let policies = match client {
            Client::Surge => {
                let raw_profile = SurgeProfile::parse(raw_profile_content)?;
                let polices = crate::core::profile::extract_policies_for_rule_provider(
                    &raw_profile.rules,
                    url_builder.sub_host()?,
                );
                if args.update {
                    self.update_surge_config(&url_builder, &polices).await?;
                }
                polices
            }
            Client::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let polices = crate::core::profile::extract_policies_for_rule_provider(
                    &raw_profile.rules,
                    url_builder.sub_host()?,
                );
                if args.update {
                    let clash_config = self.update_clash_config(&url_builder, raw_profile).await?;
                    println!("{clash_config}");
                }
                polices
            }
        };
        if matches!(client, Client::Clash) && args.update {
            return Ok(());
        }
        println!("{}", "Raw Subscription url:".to_string().green().bold());
        println!("{}", url_builder.build_subscription_url(client)?);
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

    async fn generate_url_builder(&self, args: &SubscriptionArgs) -> color_eyre::Result<UrlBuilder> {
        let SubscriptionArgs {
            client,
            server,
            reset,
            raw_sub_url,
            convertor_url,
            ..
        } = args;
        let server = server.as_ref().unwrap_or(&self.config.server).clone();
        let url_builder = if let Some(raw_sub_url) = raw_sub_url {
            UrlBuilder::new(
                server.clone(),
                self.config.secret.clone(),
                raw_sub_url.clone(),
                self.config.interval,
                self.config.strict,
            )?
        } else if let Some(convertor_url) = convertor_url {
            UrlBuilder::decode_from_convertor_url(convertor_url.clone(), &self.config.secret)?
        } else {
            let raw_sub_url = if *reset {
                self.api
                    .reset_raw_sub_url(self.config.service_config.base_url.clone())
                    .await?
            } else {
                self.api
                    .get_raw_sub_url(self.config.service_config.base_url.clone(), *client)
                    .await?
            };
            UrlBuilder::new(
                server,
                self.config.secret.clone(),
                raw_sub_url,
                self.config.interval,
                self.config.strict,
            )?
        };
        Ok(url_builder)
    }

    async fn update_surge_config(&self, url_builder: &UrlBuilder, policies: &[Policy]) -> color_eyre::Result<()> {
        let surge_config = SurgeConfig::try_new()?;
        surge_config.update_surge_config(url_builder).await?;
        surge_config
            .update_surge_sub_logs_url(url_builder.build_sub_logs_url(&self.config.secret)?)
            .await?;
        surge_config.update_surge_rule_providers(url_builder, policies).await?;
        Ok(())
    }

    async fn update_clash_config(
        &self,
        url_builder: &UrlBuilder,
        raw_profile: ClashProfile,
    ) -> color_eyre::Result<String> {
        let mut template = ClashProfile::template()?;
        template.merge(raw_profile, &self.config.secret)?;
        template.optimize(url_builder)?;
        let clash_config = ClashRenderer::render_profile(&template)?;
        Ok(clash_config)
    }
}

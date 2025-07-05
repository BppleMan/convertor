use crate::client::Client;
use crate::config::convertor_config::ConvertorConfig;
use crate::profile;
use crate::profile::clash_profile::ClashProfile;
use crate::profile::renderer::clash_renderer::ClashRenderer;
use crate::profile::renderer::surge_renderer::SurgeRenderer;
use crate::profile::surge_profile::SurgeProfile;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::subscription_command::{SubCommonArgs, SubscriptionCommand};
use crate::subscription::url_builder::UrlBuilder;

pub struct SubscriptionService {
    pub config: ConvertorConfig,
    pub api: BosLifeApi,
}

impl SubscriptionService {
    pub async fn execute(&mut self, command: SubscriptionCommand) -> color_eyre::Result<UrlBuilder> {
        let SubCommonArgs { client, .. } = command.args();
        let client = *client;
        let url_builder = self.generate_url_builder(&command).await?;
        let raw_sub_url = url_builder.build_subscription_url(client)?;
        let raw_profile_content = self.api.get_raw_profile(raw_sub_url, client).await?;
        let policies = match client {
            Client::Surge => {
                let raw_profile = SurgeProfile::parse(raw_profile_content)?;
                profile::core::extract_policies_for_rule_provider(&raw_profile.rules, url_builder.sub_host()?)
            }
            Client::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                profile::core::extract_policies_for_rule_provider(&raw_profile.rules, url_builder.sub_host()?)
            }
        };
        println!("Raw Subscription url:");
        println!("{}", url_builder.build_subscription_url(client)?);
        println!("Convertor url:");
        println!("{}", url_builder.build_convertor_url(client)?);
        for policy in policies {
            match client {
                Client::Surge => println!("{}", SurgeRenderer::render_policy_for_provider(&policy)),
                Client::Clash => println!("{}", ClashRenderer::render_policy_for_provider(&policy)),
            }
            println!("{}", url_builder.build_rule_provider_url(client, &policy)?)
        }
        Ok(url_builder)
    }

    async fn generate_url_builder(&self, command: &SubscriptionCommand) -> color_eyre::Result<UrlBuilder> {
        let SubCommonArgs { client, server } = command.args();
        let client = *client;
        let server = server.clone().unwrap_or_else(|| self.config.server.clone());
        let url_builder = match command {
            SubscriptionCommand::Get(_) => {
                let raw_url = self
                    .api
                    .get_raw_sub_url(self.config.service_config.base_url.clone(), client)
                    .await?;
                UrlBuilder::new(server, self.config.secret.clone(), raw_url)?
            }
            SubscriptionCommand::Update { reset_token, .. } => {
                let raw_url = if *reset_token {
                    self.api
                        .reset_raw_sub_url(self.config.service_config.base_url.clone())
                        .await?
                } else {
                    self.api
                        .get_raw_sub_url(self.config.service_config.base_url.clone(), client)
                        .await?
                };
                UrlBuilder::new(server, self.config.secret.clone(), raw_url)?
            }
            SubscriptionCommand::Encode {
                raw_subscription_url, ..
            } => UrlBuilder::new(server, self.config.secret.clone(), raw_subscription_url.clone())?,
            SubscriptionCommand::Decode { convertor_url, .. } => {
                UrlBuilder::decode_from_convertor_url(convertor_url.clone(), &self.config.secret)?
            }
        };
        Ok(url_builder)
    }

    pub async fn update_surge_config() {}
}

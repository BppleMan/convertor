use crate::client::Client;
use crate::config::convertor_config::ConvertorConfig;
use crate::config::surge_config::SurgeConfig;
use crate::profile;
use crate::profile::clash_profile::ClashProfile;
use crate::profile::surge_profile::SurgeProfile;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::subscription_command::SubscriptionCommand;
use crate::subscription::url_builder::UrlBuilder;
use url::Url;

pub struct SubscriptionService {
    pub config: ConvertorConfig,
    pub service: BosLifeApi,
}

impl SubscriptionService {
    pub async fn execute(
        &mut self,
        command: SubscriptionCommand,
        server: Option<Url>,
        client: Client,
    ) -> color_eyre::Result<UrlBuilder> {
        let server = server.unwrap_or_else(|| self.config.server.clone());
        let url_builder = match command {
            SubscriptionCommand::Get => {
                let raw_url = self.service.get_raw_subscription_url().await?;
                UrlBuilder::new(server, self.config.secret.clone(), raw_url)?
            }
            SubscriptionCommand::Update { refresh_token } => {
                let raw_url = if refresh_token {
                    self.service.reset_raw_subscription_url().await?
                } else {
                    self.service.get_raw_subscription_url().await?
                };
                let url_builder = UrlBuilder::new(server, self.config.secret.clone(), raw_url)?;
                let surge_config = SurgeConfig::try_new()?;
                // surge_config.update_surge_config(&url_builder).await?;
                // surge_config.update_surge_rule_set(&url_builder).await?;
                url_builder
            }
            SubscriptionCommand::Encode { raw_subscription_url } => {
                let raw_url = Url::parse(&raw_subscription_url)?;
                UrlBuilder::new(server, self.config.secret.clone(), raw_url)?
            }
            SubscriptionCommand::Decode { convertor_url } => {
                UrlBuilder::decode_from_convertor_url(convertor_url, &self.config.secret)?
            }
        };
        let raw_profile_content = self
            .service
            .get_raw_profile(url_builder.build_subscription_url(client)?, client)
            .await?;
        match client {
            Client::Surge => {
                let raw_profile = SurgeProfile::parse(raw_profile_content)?;
                let policies = profile::core::extract_policies(&raw_profile.rules);
                println!("{:#?}", policies);
            }
            Client::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let policies = profile::core::extract_policies(&raw_profile.rules);
                println!("{:#?}", policies);
            }
        }
        // println!("Raw Subscription URL:\n{}", url_builder.build_subscription_url(client)?);
        // println!("Convertor URL:\n{}", url_builder.build_convertor_url(client)?);
        // for rsp in PolicyPreset::all() {
        //     let rule_set_url = url_builder.build_rule_set_url(&flag, rsp)?;
        //     println!("{}:\n{}", rsp.section_name(), rule_set_url);
        // }
        Ok(url_builder)
    }

    pub async fn update_surge_config() {}
}

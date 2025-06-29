use crate::config::convertor_config::ConvertorConfig;
use crate::config::surge_config::SurgeConfig;
use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::subscription_command::SubscriptionCommand;
use crate::subscription::url_builder::UrlBuilder;
use reqwest::Client;
use url::Url;

pub struct SubscriptionService {
    pub config: ConvertorConfig,
}

impl SubscriptionService {
    pub async fn execute(
        &self,
        command: SubscriptionCommand,
        server: Option<Url>,
        flag: String,
    ) -> color_eyre::Result<UrlBuilder> {
        let subscription_api = BosLifeApi::new(Client::new(), self.config.service_config.clone());
        let server = server.unwrap_or_else(|| self.config.server.clone());
        let url_builder = match command {
            SubscriptionCommand::Get => {
                let raw_url = subscription_api.get_raw_subscription_url().await?;
                UrlBuilder::new(server, self.config.secret.clone(), raw_url)?
            }
            SubscriptionCommand::Update { refresh_token } => {
                let raw_url = if refresh_token {
                    subscription_api.reset_raw_subscription_url().await?
                } else {
                    subscription_api.get_raw_subscription_url().await?
                };
                let url_builder = UrlBuilder::new(server, self.config.secret.clone(), raw_url)?;
                let surge_config = SurgeConfig::try_new()?;
                surge_config.update_surge_config(&url_builder).await?;
                surge_config.update_surge_rule_set(&url_builder).await?;
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
        println!("Raw Subscription URL:\n{}", url_builder.build_subscription_url(&flag)?);
        println!("Convertor URL:\n{}", url_builder.build_convertor_url(&flag)?);
        println!("Proxy Provider:\n{}", url_builder.build_proxy_provider_url(&flag)?);
        for rsp in RuleSetPolicy::all() {
            let rule_set_url = url_builder.build_rule_set_url(&flag, rsp)?;
            println!("{}:\n{}", rsp.name(), rule_set_url);
        }
        Ok(url_builder)
    }

    pub async fn update_surge_config() {}
}

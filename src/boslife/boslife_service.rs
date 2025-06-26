use crate::boslife::boslife_command::BosLifeCommand;
use crate::boslife::boslife_credential::BosLifeCredential;
use crate::config::convertor_config::ConvertorConfig;
use crate::config::service_config::ServiceConfig;
use crate::config::surge_config::SurgeConfig;
use crate::config::url_builder::UrlBuilder;
use crate::profile::RuleSetPolicy;
use crate::service::service_api::ServiceApi;
use crate::service::subscription_log::SubscriptionLog;
use moka::future::Cache;
use reqwest::{Client, Url};

#[derive(Clone)]
pub struct BosLifeService {
    pub config: ConvertorConfig<BosLifeCredential>,
    pub client: Client,
    pub cached_string: Cache<String, String>,
    pub cached_subscription_logs: Cache<String, Vec<SubscriptionLog>>,
}

impl BosLifeService {
    pub fn new(
        client: Client,
        config: ConvertorConfig<BosLifeCredential>,
    ) -> Self {
        let cached_string = Cache::builder()
            .max_capacity(10)
            .time_to_live(std::time::Duration::from_secs(60 * 10)) // 10 minutes
            .build();
        let cached_subscription_logs = Cache::builder()
            .max_capacity(10)
            .time_to_live(std::time::Duration::from_secs(60 * 10)) // 10 minutes
            .build();
        Self {
            config,
            client,
            cached_string,
            cached_subscription_logs,
        }
    }

    pub async fn execute(
        &self,
        command: BosLifeCommand,
        server: Option<String>,
        flag: String,
    ) -> color_eyre::Result<UrlBuilder> {
        let url_builder = match command {
            BosLifeCommand::Get => {
                let server = server.map(|s| Url::parse(&s)).transpose()?;
                self.get_subscription_url(server).await
            }
            BosLifeCommand::Update { refresh_token } => {
                let server = server.map(|s| Url::parse(&s)).transpose()?;
                self.update_surge_config(server, refresh_token).await
            }
            BosLifeCommand::Encode {
                raw_subscription_url,
            } => {
                let server = server.map(|s| Url::parse(&s)).transpose()?;
                self.encode_subscription_url(server, raw_subscription_url)
                    .await
            }
            BosLifeCommand::Decode { convertor_url } => {
                self.decode_convertor_url(convertor_url).await
            }
        }?;
        println!(
            "Raw Subscription URL:\n{}",
            url_builder.build_subscription_url(&flag)?
        );
        println!(
            "Convertor URL:\n{}",
            url_builder.build_convertor_url(&flag)?
        );
        println!(
            "Proxy Provider:\n{}",
            url_builder.build_proxy_provider_url(&flag)?
        );
        for rsp in RuleSetPolicy::all() {
            let rule_set_url = url_builder.build_rule_set_url(&flag, rsp)?;
            println!("{}:\n{}", rsp.name(), rule_set_url);
        }
        Ok(url_builder)
    }

    pub async fn get_subscription_url(
        &self,
        server: Option<Url>,
    ) -> color_eyre::Result<UrlBuilder> {
        let auth_token = self.login().await?;
        let raw_subscription_url =
            self.get_raw_subscription_url(&auth_token).await?;
        let convertor_url = UrlBuilder::new(
            server.unwrap_or(self.config.server.clone()),
            &self.config.secret,
            raw_subscription_url,
        )?;
        Ok(convertor_url)
    }

    pub async fn update_surge_config(
        &self,
        server: Option<Url>,
        refresh_token: bool,
    ) -> color_eyre::Result<UrlBuilder> {
        let auth_token = self.login().await?;
        let raw_subscription_url = if refresh_token {
            self.reset_raw_subscription_url(&auth_token).await?
        } else {
            self.get_raw_subscription_url(&auth_token).await?
        };

        let convertor_url = UrlBuilder::new(
            server.unwrap_or(self.config.server.clone()),
            &self.config.secret,
            raw_subscription_url,
        )?;
        let surge_config = SurgeConfig::try_new()?;

        surge_config.update_surge_config(&convertor_url).await?;
        surge_config.update_surge_rule_set(&convertor_url).await?;

        Ok(convertor_url)
    }

    pub async fn encode_subscription_url(
        &self,
        server: Option<Url>,
        raw_subscription_url: String,
    ) -> color_eyre::Result<UrlBuilder> {
        let raw_subscription_url = Url::parse(&raw_subscription_url)?;
        let convertor_url = UrlBuilder::new(
            server.unwrap_or(self.config.server.clone()),
            &self.config.secret,
            raw_subscription_url,
        )?;
        Ok(convertor_url)
    }

    pub(crate) async fn decode_convertor_url(
        &self,
        raw_convertor_url: String,
    ) -> color_eyre::Result<UrlBuilder> {
        let convertor_url = UrlBuilder::decode_from_convertor_url(
            raw_convertor_url,
            &self.config.secret,
        )?;
        Ok(convertor_url)
    }
}

impl ServiceApi for BosLifeService {
    type Cred = BosLifeCredential;

    fn config(&self) -> &ServiceConfig<BosLifeCredential> {
        &self.config.service_config
    }

    fn client(&self) -> &Client {
        &self.client
    }

    fn cached_auth_token(&self) -> &Cache<String, String> {
        &self.cached_string
    }

    fn cached_profile(&self) -> &Cache<String, String> {
        &self.cached_string
    }

    fn cached_subscription_logs(&self) -> &Cache<String, Vec<SubscriptionLog>> {
        &self.cached_subscription_logs
    }
}

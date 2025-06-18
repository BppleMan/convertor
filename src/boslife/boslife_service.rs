use crate::boslife::boslife_command::BosLifeCommand;
use crate::boslife::boslife_credential::BosLifeCredential;
use crate::config::convertor_config::ConvertorConfig;
use crate::config::service_config::ServiceConfig;
use crate::config::surge_config::{RuleSetType, SurgeConfig};
use crate::convertor_url::ConvertorUrl;
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
    ) -> color_eyre::Result<ConvertorUrl> {
        let convertor_url = match command {
            BosLifeCommand::Get { server } => {
                let server = server.map(|s| Url::parse(&s)).transpose()?;
                self.get_subscription_url(server).await
            }
            BosLifeCommand::Update {
                server,
                refresh_token,
            } => {
                let server = server.map(|s| Url::parse(&s)).transpose()?;
                self.update_surge_config(server, refresh_token).await
            }
            BosLifeCommand::Encode {
                server,
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
            "Raw Subscription URL:\n\t{}",
            convertor_url.build_subscription_url("surge")?
        );
        println!(
            "Convertor URL:\n\t{}",
            convertor_url.build_convertor_url("surge")?
        );
        for rst in RuleSetType::all() {
            let rule_set_url = convertor_url.build_rule_set_url(rst)?;
            println!("Rule Set URL ({}):\n\t{}", rst.name(), rule_set_url);
        }
        Ok(convertor_url)
    }

    pub async fn get_subscription_url(
        &self,
        server: Option<Url>,
    ) -> color_eyre::Result<ConvertorUrl> {
        let auth_token = self.login().await?;
        let raw_subscription_url =
            self.get_raw_subscription_url(&auth_token).await?;
        let convertor_url = ConvertorUrl::new(
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
    ) -> color_eyre::Result<ConvertorUrl> {
        let auth_token = self.login().await?;
        let raw_subscription_url = if refresh_token {
            self.reset_raw_subscription_url(&auth_token).await?
        } else {
            self.get_raw_subscription_url(&auth_token).await?
        };

        let convertor_url = ConvertorUrl::new(
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
    ) -> color_eyre::Result<ConvertorUrl> {
        let raw_subscription_url = Url::parse(&raw_subscription_url)?;
        let convertor_url = ConvertorUrl::new(
            server.unwrap_or(self.config.server.clone()),
            &self.config.secret,
            raw_subscription_url,
        )?;
        Ok(convertor_url)
    }

    pub(crate) async fn decode_convertor_url(
        &self,
        raw_convertor_url: String,
    ) -> color_eyre::Result<ConvertorUrl> {
        let convertor_url = ConvertorUrl::decode_from_convertor_url(
            raw_convertor_url,
            &self.config.secret,
        )?;
        Ok(convertor_url)
    }
}

impl ServiceApi for BosLifeService {
    type Cred = BosLifeCredential;

    fn config(&self) -> &ServiceConfig {
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

    fn get_credential(&self) -> &BosLifeCredential {
        &self.config.service_credential
    }
}

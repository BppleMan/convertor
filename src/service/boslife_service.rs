use crate::op;
use crate::op::OpItem;
use crate::service::service_api::ServiceApi;
use crate::service::service_config::{ConfigApi, ServiceConfig};
use crate::service::subscription_log::SubscriptionLog;
use color_eyre::eyre::eyre;
use moka::future::Cache;
use reqwest::Client;
use tracing::info;

const CACHED_CREDENTIAL_KEY: &str = "CACHED_CREDENTIAL";

pub struct BosLifeService {
    pub config: ServiceConfig,
    pub client: Client,
    pub cached_string: Cache<String, String>,
    pub cached_subscription_logs: Cache<String, Vec<SubscriptionLog>>,
    pub cached_credential: Cache<String, OpItem>,
}

impl BosLifeService {
    pub fn new(client: Client) -> Self {
        let config = ServiceConfig {
            base_url: "https://www.blnew.com",
            prefix_path: "/proxy",
            one_password_key: "pkrtud2bg5clrfmtm254quskzu",
            login_api: ConfigApi {
                api_path: "/passport/auth/login",
                json_path: "$.data.auth_data",
            },
            reset_subscription_url: ConfigApi {
                api_path: "/user/resetSecurity",
                json_path: "$.data",
            },
            get_subscription_url: ConfigApi {
                api_path: "/user/getSubscribe",
                json_path: "$.data.subscribe_url",
            },
            get_subscription_log: ConfigApi {
                api_path: "/user/stat/getSubscribeLog",
                json_path: "$.data",
            },
        };
        let cached_string = Cache::builder()
            .max_capacity(10)
            .time_to_live(std::time::Duration::from_secs(60 * 10)) // 10 minutes
            .build();
        let cached_subscription_logs = Cache::builder()
            .max_capacity(10)
            .time_to_live(std::time::Duration::from_secs(60 * 10)) // 10 minutes
            .build();
        let cached_credential = Cache::builder()
            .max_capacity(10)
            .time_to_live(std::time::Duration::from_secs(60 * 10)) // 10 minutes
            .build();
        Self {
            config,
            client,
            cached_string,
            cached_subscription_logs,
            cached_credential,
        }
    }
}

impl ServiceApi for BosLifeService {
    fn config(&self) -> &ServiceConfig {
        &self.config
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
        info!(
            "获取缓存的订阅日志: {}",
            self.cached_subscription_logs.entry_count()
        );
        &self.cached_subscription_logs
    }

    async fn get_credential(&self) -> color_eyre::Result<OpItem> {
        self.cached_credential
            .try_get_with(CACHED_CREDENTIAL_KEY.to_string(), async {
                info!("尝试从环境变量中获取 BosLife 凭据");
                let username = std::env::var("BOSLIFE_USERNAME").ok();
                let password = std::env::var("BOSLIFE_PASSWORD").ok();
                if let (Some(username), Some(password)) = (username, password) {
                    return Ok(OpItem { username, password });
                }
                info!("尝试从 1Password 中获取 BosLife 凭据");
                color_eyre::Result::<OpItem>::Ok(
                    op::get_item(self.config().one_password_key).await?,
                )
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

#[cfg(test)]
mod tests {
    use crate::service::boslife_service::BosLifeService;
    use crate::service::service_api::ServiceApi;
    use crate::service::subscription_log::SubscriptionLog;

    #[tokio::test]
    async fn test_login() -> color_eyre::Result<()> {
        color_eyre::install()?;
        let client = reqwest::Client::new();
        let service = BosLifeService::new(client);
        let auth_token = service.login().await?;
        println!("{}", auth_token);

        let subscription_url =
            service.get_subscription_url(&auth_token).await?;
        println!("{}", subscription_url);

        let subscription_logs: Vec<SubscriptionLog> =
            service.get_subscription_log(&auth_token).await?;
        println!("{:#?}", subscription_logs);
        Ok(())
    }
}

use crate::airport::airport_api::AirportApi;
use crate::airport::airport_config::{AirportConfig, ConfigApi};
use reqwest::Client;

pub struct BosLifeService {
    pub config: AirportConfig,
    pub client: Client,
}

impl BosLifeService {
    pub fn new(client: Client) -> Self {
        let config = AirportConfig {
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
        Self { config, client }
    }
}

impl AirportApi for BosLifeService {
    fn config(&self) -> &AirportConfig {
        &self.config
    }

    fn client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use crate::airport::airport_api::AirportApi;
    use crate::airport::boslife_service::BosLifeService;
    use crate::airport::subscription_log::SubscriptionLog;

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

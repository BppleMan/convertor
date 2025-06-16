use crate::config::surge_config::SurgeConfig;
use crate::convertor_url::ConvertorUrl;
use crate::op;
use crate::op::OpItem;
use crate::service::service_api::ServiceApi;
use crate::service::service_config::{ConfigApi, ServiceConfig};
use crate::service::subscription_log::SubscriptionLog;
use clap::Subcommand;
use color_eyre::eyre::eyre;
use color_eyre::Report;
use moka::future::Cache;
use reqwest::{Client, Url};
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
            one_password_key: "BosLife",
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
                Ok::<OpItem, Report>(
                    op::get_item(self.config().one_password_key).await?,
                )
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

#[derive(Debug, Subcommand)]
pub enum BosLifeServiceSubscription {
    /// 从 boslife 获取订阅地址
    Get {
        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, default_value = "http://127.0.0.1:8001")]
        server: String,

        /// boslife 的用户名
        #[arg(short)]
        username: Option<String>,

        /// boslife 的密码
        #[arg(short)]
        password: Option<String>,
    },
    /// 从 boslife 更新订阅地址
    Update {
        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, long)]
        server: String,

        /// 是否刷新 boslife token
        #[arg(short, long, default_value = "false")]
        refresh_token: bool,

        /// boslife 的用户名
        #[arg(short)]
        username: Option<String>,

        /// boslife 的密码
        #[arg(short)]
        password: Option<String>,
    },
    /// 根据 boslife 的订阅地址编码为 convertor 的订阅地址
    Encode {
        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, default_value = "http://127.0.0.1:8001")]
        server: String,
        /// boslife 的订阅地址
        #[arg(short, long = "url")]
        raw_subscription_url: String,
    },
    /// 根据 convertor 的订阅地址解码为 boslife 的订阅地址
    Decode {
        /// convertor 的订阅地址
        #[arg(short, long = "url")]
        convertor_url: String,
    },
}

impl BosLifeServiceSubscription {
    pub async fn execute(
        self,
        service: BosLifeService,
    ) -> color_eyre::Result<ConvertorUrl> {
        let convertor_url = match self {
            BosLifeServiceSubscription::Get {
                server,
                username,
                password,
            } => {
                Self::get_subscription_url(service, server, username, password)
                    .await
            }
            BosLifeServiceSubscription::Update {
                server,
                refresh_token,
                username,
                password,
            } => {
                Self::update_surge_config(
                    service,
                    server,
                    refresh_token,
                    username,
                    password,
                )
                .await
            }
            BosLifeServiceSubscription::Encode {
                server,
                raw_subscription_url,
            } => {
                Self::encode_subscription_url(server, raw_subscription_url)
                    .await
            }
            BosLifeServiceSubscription::Decode { convertor_url } => {
                Self::decode_convertor_url(convertor_url).await
            }
        }?;
        println!(
            "Raw Subscription URL: {}",
            convertor_url.build_subscription_url("surge")?
        );
        println!(
            "Convertor URL: {}",
            convertor_url.build_convertor_url("surge")?
        );
        Ok(convertor_url)
    }

    pub(super) async fn get_subscription_url(
        service: BosLifeService,
        server: String,
        username: Option<String>,
        password: Option<String>,
    ) -> color_eyre::Result<ConvertorUrl> {
        let credential = username.and_then(|username| {
            password.and_then(|password| Some(OpItem { username, password }))
        });
        let auth_token = service.login(credential).await?;
        let raw_subscription_url =
            service.get_raw_subscription_url(&auth_token).await?;
        let convertor_url = ConvertorUrl::new(server, raw_subscription_url)?;
        Ok(convertor_url)
    }

    pub(super) async fn update_surge_config(
        service: BosLifeService,
        server: String,
        refresh_token: bool,
        username: Option<String>,
        password: Option<String>,
    ) -> color_eyre::Result<ConvertorUrl> {
        let credential = username.and_then(|username| {
            password.and_then(|password| Some(OpItem { username, password }))
        });
        let auth_token = service.login(credential).await?;
        let raw_subscription_url = if refresh_token {
            service.reset_raw_subscription_url(&auth_token).await?
        } else {
            service.get_raw_subscription_url(&auth_token).await?
        };

        let convertor_url = ConvertorUrl::new(server, raw_subscription_url)?;
        let surge_config = SurgeConfig::try_new()?;

        surge_config.update_surge_config(&convertor_url).await?;
        surge_config.update_surge_rule_set(&convertor_url).await?;

        Ok(convertor_url)
    }

    pub(super) async fn encode_subscription_url(
        server: String,
        raw_subscription_url: String,
    ) -> color_eyre::Result<ConvertorUrl> {
        let raw_subscription_url = Url::parse(&raw_subscription_url)?;
        let convertor_url = ConvertorUrl::new(server, raw_subscription_url)?;
        Ok(convertor_url)
    }

    pub(super) async fn decode_convertor_url(
        raw_convertor_url: String,
    ) -> color_eyre::Result<ConvertorUrl> {
        let convertor_url =
            ConvertorUrl::decode_from_convertor_url(raw_convertor_url)?;
        Ok(convertor_url)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::surge_config::RuleSetType;
    use crate::encrypt::decrypt;
    use crate::op::get_convertor_secret;
    use crate::service::boslife_service::{
        BosLifeService, BosLifeServiceSubscription,
    };
    use crate::service::service_api::ServiceApi;
    use crate::service::subscription_log::SubscriptionLog;

    #[tokio::test]
    async fn test_login() -> color_eyre::Result<()> {
        color_eyre::install()?;
        let client = reqwest::Client::new();
        let service = BosLifeService::new(client);
        let auth_token = service.login(None).await?;
        println!("{}", auth_token);

        let subscription_url =
            service.get_raw_subscription_url(&auth_token).await?;
        println!("{}", subscription_url);

        let subscription_logs: Vec<SubscriptionLog> =
            service.get_subscription_log(&auth_token).await?;
        println!("{:#?}", subscription_logs);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_subscription_url() -> color_eyre::Result<()> {
        color_eyre::install()?;

        let client = reqwest::Client::new();
        let service = BosLifeService::new(client);
        let convertor_url = BosLifeServiceSubscription::get_subscription_url(
            service,
            "http://127.0.0.1:8001".to_string(),
            None,
            None,
        )
        .await?;
        let raw_convertor_url = convertor_url.build_convertor_url("surge")?;

        let convertor_url = BosLifeServiceSubscription::decode_convertor_url(
            raw_convertor_url.to_string(),
        )
        .await?;

        assert_eq!(convertor_url, convertor_url);

        Ok(())
    }

    #[tokio::test]
    async fn test_subscription_and_rule_set() -> color_eyre::Result<()> {
        color_eyre::install()?;

        let client = reqwest::Client::new();
        let service = BosLifeService::new(client);
        let convertor_url = BosLifeServiceSubscription::get_subscription_url(
            service,
            "http://127.0.0.1:8001".to_string(),
            None,
            None,
        )
        .await?;

        let raw_convertor_url = convertor_url.build_convertor_url("surge")?;
        println!("{}", raw_convertor_url);
        for rst in RuleSetType::all() {
            let rule_set_url = convertor_url.build_rule_set_url(rst)?;
            println!("{}", rule_set_url);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_decrypt() -> color_eyre::Result<()> {
        color_eyre::install()?;
        let secret = get_convertor_secret()?;
        println!("{}", secret);
        let str = decrypt(secret.as_bytes(), "zJquxjk/pXgL5c4q:Pu+5AyJpf/iwH4mHQy0W8dz+ONpZx+94UYERvrhLr5xokl40ada8nSauiJnnSsM9ejHo2idldxS21fIROQMyPFhPDX11KNYInasNHUhLaHpUbW7e4WNs7dnbaSDMp5npALRBPvVT9znkjHf0")?;
        println!("{}", str);
        Ok(())
    }
}

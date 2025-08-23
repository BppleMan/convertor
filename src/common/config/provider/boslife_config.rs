use crate::common::config::provider::{ApiConfig, CredentialConfig};
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::request::RequestConfig;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BosLifeConfig {
    pub sub_url: Url,
    #[serde(default)]
    pub request: Option<RequestConfig>,
    pub credential: CredentialConfig,
}

impl BosLifeConfig {
    pub fn template() -> Self {
        Self {
            sub_url: Url::parse("http://127.0.0.1:8080/subscription?token=bppleman").expect("不合法的订阅地址"),
            request: Some(RequestConfig::template()),
            credential: CredentialConfig {
                username: "optional[boslife.username]".to_string(),
                password: "optional[boslife.password]".to_string(),
            },
        }
    }

    pub fn build_raw_sub_url(&self, client: ProxyClient) -> Url {
        let mut url = self.sub_url.clone();
        // BosLife 的字段是 `flag` 不可改为client
        url.query_pairs_mut().append_pair("flag", client.into());
        url
    }
}

impl BosLifeConfig {
    pub fn login_url_api(&self) -> ApiConfig {
        ApiConfig {
            api: "https://www.blnew.com/proxy/passport/auth/login",
            json_path: "$.data.auth_data",
        }
    }

    pub fn get_sub_url_api(&self) -> ApiConfig {
        ApiConfig {
            api: "https://www.blnew.com/proxy/user/getSubscribe",
            json_path: "$.data.subscribe_url",
        }
    }

    pub fn reset_sub_url_api(&self) -> ApiConfig {
        ApiConfig {
            api: "https://www.blnew.com/proxy/user/resetSecurity",
            json_path: "$.data",
        }
    }

    pub fn get_sub_logs_url_api(&self) -> ApiConfig {
        ApiConfig {
            api: "https://www.blnew.com/proxy/user/stat/getSubscribeLog",
            json_path: "$.data",
        }
    }
}

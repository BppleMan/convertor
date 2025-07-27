use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::request::RequestConfig;
use clap::ValueEnum;
use dispatch_map::dispatch_pattern;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use thiserror::Error;
use url::Url;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubProvider {
    #[default]
    BosLife,
}

impl SubProvider {
    pub fn providers() -> [Self; 1] {
        [SubProvider::BosLife]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SubProvider::BosLife => "boslife",
        }
    }
}

impl FromStr for SubProvider {
    type Err = ParseProviderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "boslife" => Ok(SubProvider::BosLife),
            _ => Err(ParseProviderError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SubProviderConfig {
    BosLife(BosLifeConfig),
}

dispatch_pattern! {
    SubProvider::BosLife => SubProviderConfig::BosLife(BosLifeConfig),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BosLifeConfig {
    pub uni_sub_url: Url,
    #[serde(default)]
    pub request: Option<RequestConfig>,
    pub credential: CredentialConfig,
    pub api_host: Url,
    pub api_prefix: String,
    pub login_api: ApiConfig,
    pub reset_sub_url_api: ApiConfig,
    pub get_sub_url_api: ApiConfig,
    pub get_sub_logs_api: ApiConfig,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ApiConfig {
    pub path: String,
    pub json_path: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CredentialConfig {
    pub username: String,
    pub password: String,
}

impl SubProviderConfig {
    pub fn uni_sub_url(&self) -> &Url {
        match self {
            SubProviderConfig::BosLife(config) => &config.uni_sub_url,
        }
    }

    pub fn request(&self) -> Option<&RequestConfig> {
        match self {
            SubProviderConfig::BosLife(config) => config.request.as_ref(),
        }
    }

    pub fn credential(&self) -> &CredentialConfig {
        match self {
            SubProviderConfig::BosLife(config) => &config.credential,
        }
    }

    pub fn build_raw_sub_url(&self, client: ProxyClient) -> Url {
        match self {
            SubProviderConfig::BosLife(config) => config.build_raw_sub_url(client),
        }
    }

    pub fn build_login_url(&self) -> Url {
        match self {
            SubProviderConfig::BosLife(config) => config.build_login_url(),
        }
    }

    pub fn build_get_sub_url(&self) -> Url {
        match self {
            SubProviderConfig::BosLife(config) => config.build_get_sub_url(),
        }
    }

    pub fn build_reset_sub_url(&self) -> Url {
        match self {
            SubProviderConfig::BosLife(config) => config.build_reset_sub_url(),
        }
    }

    pub fn build_get_sub_logs_url(&self) -> Url {
        match self {
            SubProviderConfig::BosLife(config) => config.build_get_sub_logs_url(),
        }
    }
}

impl Display for SubProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubProvider::BosLife => write!(f, "{}", self.as_str()),
        }
    }
}

impl BosLifeConfig {
    pub fn template() -> Self {
        Self {
            uni_sub_url: Url::parse("http://127.0.0.1:8080/subscription?token=bppleman").expect("不合法的订阅地址"),
            request: Some(RequestConfig::template()),
            credential: CredentialConfig {
                username: "optional:boslife.username".to_string(),
                password: "optional:boslife.password".to_string(),
            },
            api_host: Url::parse("https://www.blnew.com").expect("不合法的API地址"),
            api_prefix: "/proxy/".to_string(),
            login_api: ApiConfig {
                path: "passport/auth/login".to_string(),
                json_path: "$.data.auth_data".to_string(),
            },
            get_sub_url_api: ApiConfig {
                path: "user/getSubscribe".to_string(),
                json_path: "$.data.subscribe_url".to_string(),
            },
            reset_sub_url_api: ApiConfig {
                path: "user/resetSecurity".to_string(),
                json_path: "$.data".to_string(),
            },
            get_sub_logs_api: ApiConfig {
                path: "user/stat/getSubscribeLog".to_string(),
                json_path: "$.data".to_string(),
            },
        }
    }

    pub fn build_raw_sub_url(&self, client: ProxyClient) -> Url {
        let mut url = self.uni_sub_url.clone();
        // BosLife 的字段是 `flag` 不可改为client
        url.query_pairs_mut().append_pair("flag", client.as_str());
        url
    }

    pub fn login_path(&self) -> String {
        format!("{}{}", self.api_prefix, self.login_api.path)
    }

    pub fn build_login_url(&self) -> Url {
        let mut url = self.api_host.clone();
        url.set_path(&self.login_path());
        url
    }

    pub fn get_sub_url_path(&self) -> String {
        format!("{}{}", self.api_prefix, self.get_sub_url_api.path)
    }

    pub fn build_get_sub_url(&self) -> Url {
        let mut url = self.api_host.clone();
        url.set_path(&self.get_sub_url_path());
        url
    }

    pub fn reset_sub_url_path(&self) -> String {
        format!("{}{}", self.api_prefix, self.reset_sub_url_api.path)
    }

    pub fn build_reset_sub_url(&self) -> Url {
        let mut url = self.api_host.clone();
        url.set_path(&self.reset_sub_url_path());
        url
    }

    pub fn get_sub_logs_path(&self) -> String {
        format!("{}{}", self.api_prefix, self.get_sub_logs_api.path)
    }

    pub fn build_get_sub_logs_url(&self) -> Url {
        let mut url = self.api_host.clone();
        url.set_path(&self.get_sub_logs_path());
        url
    }
}

#[derive(Debug, Error)]
#[error("无法解析提供商: {0}")]
pub struct ParseProviderError(String);

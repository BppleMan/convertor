use crate::common::encrypt::encrypt;
use crate::config::config_error::ConfigError;
use crate::config::proxy_client::ProxyClient;
use crate::config::redis_config::RedisConfig;
use crate::config::subscription_config::SubscriptionConfig;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::path::Path;
use std::str::FromStr;
use tracing::debug;
use url::Url;

pub mod config_error;
pub mod proxy_client;
pub mod redis_config;
pub mod subscription_config;

type Result<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub secret: String,
    pub server: Url,
    pub subscription: SubscriptionConfig,
    pub redis: Option<RedisConfig>,
}

impl Config {
    // pub fn default_config(secret: impl AsRef<str>) -> Self {
    //     Self {
    //         secret: secret.as_ref().to_string(),
    //         server: Url::parse("http://127.0.0.1:8080").expect("不合法的服务器地址"),
    //         subscription: SubscriptionConfig::default(),
    //         redis: None,
    //     }
    // }

    pub fn template() -> Self {
        let secret = "bppleman".to_string();
        let server = Url::parse("http://127.0.0.1:8080").expect("不合法的服务器地址");
        let subscription = SubscriptionConfig::template();
        let redis = Some(RedisConfig::template());

        Config {
            secret,
            server,
            subscription,
            redis,
        }
    }
}

impl Config {
    pub fn search(cwd: impl AsRef<Path>, config_path: Option<impl AsRef<Path>>) -> Result<Self> {
        if let Some(path) = config_path {
            return Self::from_file(path);
        }
        let work_dir = cwd.as_ref().to_path_buf();
        if !work_dir.is_dir() {
            return Err(ConfigError::NotDirectory(work_dir.to_path_buf()));
        }
        let work_dir = work_dir.canonicalize().map_err(|e| ConfigError::PathError(e))?;
        let convertor_toml = work_dir.join("convertor.toml");
        debug!("尝试加载配置文件: {}", convertor_toml.display());
        if convertor_toml.exists() {
            Self::from_file(convertor_toml)
        } else {
            Err(ConfigError::NotFound {
                cwd: cwd.as_ref().to_path_buf(),
                config_name: "convertor.toml".to_string(),
            })
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(ConfigError::NotFile(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError(e))?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn enc_secret(&self) -> Result<String> {
        Ok(encrypt(self.secret.as_bytes(), &self.secret)?)
    }

    pub fn create_url_builder(&self, client: ProxyClient) -> Result<UrlBuilder> {
        let sub_url = self.subscription.sub_url.clone();
        let interval = self.subscription.interval;
        let strict = self.subscription.strict;
        let server = self.server.clone();
        let secret = self.secret.clone();
        let enc_secret = encrypt(secret.as_bytes(), &secret)?;
        let url_builder = UrlBuilder::new(
            secret,
            Some(enc_secret),
            client,
            server,
            sub_url,
            None,
            interval,
            strict,
        )?;
        Ok(url_builder)
    }
}

impl FromStr for Config {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).map_err(|_| std::fmt::Error)?)
    }
}

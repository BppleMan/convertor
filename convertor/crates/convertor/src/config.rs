use crate::common::encrypt::{decrypt, encrypt};
use crate::config::client_config::ClientConfig;
use crate::config::error::ConfigError;
use crate::config::provider_config::{Provider, ProviderConfig};
use crate::config::redis_config::{RedisConfig, REDIS_CONVERTOR_CONFIG_KEY};
use crate::url::url_builder::UrlBuilder;
use crate::url::url_error::UrlBuilderError;
use client_config::ProxyClient;
use redis::aio::MultiplexedConnection;
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, error, warn};
use url::Url;

pub mod client_config;
pub mod error;
pub mod provider_config;
pub mod redis_config;

type Result<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorConfig {
    pub secret: String,
    pub server: Url,
    pub redis: Option<RedisConfig>,
    pub providers: HashMap<Provider, ProviderConfig>,
    pub clients: HashMap<ProxyClient, ClientConfig>,
}

impl ConvertorConfig {
    pub fn template() -> Self {
        let secret = "bppleman".to_string();
        let server = Url::parse("http://127.0.0.1:8080").expect("不合法的服务器地址");

        let redis = Some(RedisConfig::template());

        let mut providers = HashMap::new();
        providers.insert(Provider::BosLife, ProviderConfig::boslife_template());

        let mut clients = HashMap::new();
        clients.insert(ProxyClient::Surge, ClientConfig::surge_template());
        clients.insert(ProxyClient::Clash, ClientConfig::clash_template());

        ConvertorConfig {
            secret,
            server,
            redis,
            providers,
            clients,
        }
    }

    pub async fn search_or_redis(
        cwd: impl AsRef<Path>,
        config_path: Option<impl AsRef<Path>>,
        connection: MultiplexedConnection,
    ) -> Result<Self> {
        match Self::search(cwd, config_path) {
            Ok(config) => Ok(config),
            Err(e) => {
                error!("{:?}", e);
                warn!("尝试从 Redis 获取配置");
                Self::from_redis(connection).await
            }
        }
    }

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

    pub async fn from_redis(mut connection: MultiplexedConnection) -> Result<Self> {
        let config: Option<String> = connection.get(REDIS_CONVERTOR_CONFIG_KEY).await?;
        let config = config.ok_or_else(|| ConfigError::RedisNotFound(REDIS_CONVERTOR_CONFIG_KEY.to_string()))?;
        Self::from_str(&config)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(ConfigError::NotFile(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError(e))?;
        let config: ConvertorConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn enc_secret(&self) -> Result<String> {
        Ok(encrypt(self.secret.as_bytes(), &self.secret)?)
    }

    pub fn create_url_builder(&self, client: ProxyClient, provider: Provider) -> Result<UrlBuilder> {
        let sub_url = match self.providers.get(&provider) {
            Some(provider_config) => provider_config.sub_url.clone(),
            None => return Err(UrlBuilderError::ProviderNotFound(provider))?,
        };
        let (interval, strict) = match self.clients.get(&client) {
            Some(client_config) => (client_config.interval(), client_config.strict()),
            None => return Err(UrlBuilderError::ClientNotFound(client))?,
        };
        let server = self.server.clone();
        let secret = self.secret.clone();
        let enc_secret = encrypt(secret.as_bytes(), &secret)?;
        let url_builder = UrlBuilder::new(
            secret,
            Some(enc_secret),
            client,
            provider,
            server,
            sub_url,
            None,
            interval,
            strict,
        )?;
        Ok(url_builder)
    }
}

impl FromStr for ConvertorConfig {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

impl Display for ConvertorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).map_err(|_| std::fmt::Error)?)
    }
}

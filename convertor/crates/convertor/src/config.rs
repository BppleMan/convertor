use crate::common::encrypt::{decrypt, encrypt};
use crate::common::redis::REDIS_CONVERTOR_CONFIG_KEY;
use crate::config::client_config::ClientConfig;
use crate::config::provider_config::{Provider, ProviderConfig};
use crate::config::redis_config::RedisConfig;
use crate::url::url_builder::UrlBuilder;
use client_config::ProxyClient;
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Report;
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
pub mod provider_config;
mod redis_config;

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
    ) -> color_eyre::Result<Self> {
        match Self::search(cwd, config_path) {
            Ok(config) => Ok(config),
            Err(e) => {
                error!("{:?}", e);
                warn!("尝试从 Redis 获取配置");
                Self::from_redis(connection).await
            }
        }
    }

    pub fn search(cwd: impl AsRef<Path>, config_path: Option<impl AsRef<Path>>) -> color_eyre::Result<Self> {
        if let Some(path) = config_path {
            return Self::from_file(path);
        }
        let work_dir = cwd.as_ref().to_path_buf();
        if !work_dir.is_dir() {
            return Err(eyre!("{} 不是一个目录", work_dir.display()));
        }
        let work_dir = work_dir.canonicalize()?;
        let convertor_toml = work_dir.join("convertor.toml");
        debug!("尝试加载配置文件: {}", convertor_toml.display());
        if convertor_toml.exists() {
            return Self::from_file(convertor_toml);
        }

        Err(eyre!("未找到 convertor.toml 配置文件"))
    }

    pub async fn from_redis(mut connection: MultiplexedConnection) -> color_eyre::Result<Self> {
        let config: Option<String> = connection
            .get(REDIS_CONVERTOR_CONFIG_KEY)
            .await
            .wrap_err("获取配置失败")?;
        let config = config.ok_or_else(|| eyre!("未找到配置项: {}", REDIS_CONVERTOR_CONFIG_KEY))?;
        Self::from_str(&config).wrap_err("解析配置字符串失败")
    }

    pub fn from_file(path: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(eyre!("{} 不是一个文件", path.display()));
        }
        let content = std::fs::read_to_string(path).wrap_err_with(|| eyre!("读取配置文件失败: {}", path.display()))?;
        let config: ConvertorConfig =
            toml::from_str(&content).wrap_err_with(|| format!("解析配置文件失败: {}", path.display()))?;
        Ok(config)
    }

    pub fn enc_secret(&self) -> color_eyre::Result<String> {
        Ok(encrypt(self.secret.as_bytes(), &self.secret)?)
    }

    pub fn create_url_builder(&self, client: ProxyClient, provider: Provider) -> color_eyre::Result<UrlBuilder> {
        let sub_url = match self.providers.get(&provider) {
            Some(provider_config) => provider_config.sub_url.clone(),
            None => return Err(eyre!("未找到提供商配置: [providers.{}]", provider)),
        };
        let (interval, strict) = match self.clients.get(&client) {
            Some(client_config) => (client_config.interval(), client_config.strict()),
            None => return Err(eyre!("未找到代理客户端配置: [clients.{}]", client)),
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

    pub fn validate_enc_secret(&self, enc_secret: &str) -> color_eyre::Result<()> {
        if enc_secret.is_empty() {
            return Err(eyre!("加密后的 secret 不能为空"));
        }
        let decrypted = decrypt(self.secret.as_bytes(), enc_secret).wrap_err("无法解密 secret")?;
        if decrypted != self.secret {
            return Err(eyre!("解密后的 secret 不匹配"));
        }
        Ok(())
    }
}

impl FromStr for ConvertorConfig {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).wrap_err("解析配置字符串失败")
    }
}

impl Display for ConvertorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).expect("无法序列化 ConvertorConfig"))
    }
}

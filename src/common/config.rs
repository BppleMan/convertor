use crate::common::config::proxy_client::{ClashConfig, ProxyClientConfig, SurgeConfig};
use crate::common::config::sub_provider::{BosLifeConfig, SubProvider, SubProviderConfig};
use crate::common::encrypt::{decrypt, encrypt};
use crate::common::redis_info::REDIS_CONVERTOR_CONFIG_KEY;
use crate::core::url_builder::UrlBuilder;
use color_eyre::Report;
use color_eyre::eyre::{WrapErr, eyre};
use dispatch_map::DispatchMap;
use proxy_client::ProxyClient;
use redis::AsyncTypedCommands;
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::path::Path;
use std::str::FromStr;
use tracing::{error, warn};
use url::Url;

pub mod config_cmd;
pub mod proxy_client;
pub mod request;
pub mod sub_provider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorConfig {
    pub secret: String,
    pub server: Url,
    pub providers: DispatchMap<SubProvider, SubProviderConfig>,
    #[serde(default)]
    pub clients: DispatchMap<ProxyClient, ProxyClientConfig>,
}

impl ConvertorConfig {
    pub fn template() -> Self {
        let mut providers = DispatchMap::default();
        providers.insert(
            SubProvider::BosLife,
            SubProviderConfig::BosLife(BosLifeConfig::template()),
        );

        let mut clients = DispatchMap::default();
        clients.insert(ProxyClient::Surge, ProxyClientConfig::Surge(SurgeConfig::template()));
        clients.insert(ProxyClient::Clash, ProxyClientConfig::Clash(ClashConfig::template()));

        ConvertorConfig {
            secret: "bppleman".to_string(),
            server: Url::parse("http://127.0.0.1:8080").expect("不合法的服务器地址"),
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
        let mut current = cwd.as_ref().to_path_buf().canonicalize()?;
        if !current.is_dir() {
            return Err(eyre!("{} 不是一个目录", current.display()));
        }
        loop {
            let file = current.join("convertor.toml");
            if file.exists() {
                return Self::from_file(file);
            }
            if !current.pop() {
                break;
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let home_dir = std::env::var("HOME")?;
            let convertor_toml = Path::new(&home_dir).join(".convertor").join("convertor.toml");
            if convertor_toml.exists() {
                return Self::from_file(convertor_toml);
            }
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

    pub fn server_addr(&self) -> color_eyre::Result<String> {
        self.server
            .host_str()
            .and_then(|host| self.server.port().map(|port| format!("{host}:{port}")))
            .ok_or_else(|| eyre!("服务器地址无效"))
    }

    pub fn server_origin(&self) -> color_eyre::Result<String> {
        self.server
            .origin()
            .ascii_serialization()
            .parse()
            .wrap_err("服务器地址无效")
    }

    pub fn enc_secret(&self) -> color_eyre::Result<String> {
        Ok(encrypt(self.secret.as_bytes(), &self.secret)?)
    }

    pub fn create_url_builder(&self, client: ProxyClient, provider: SubProvider) -> color_eyre::Result<UrlBuilder> {
        let uni_sub_url = match self.providers.get(&provider) {
            Some(SubProviderConfig::BosLife(provider_config)) => provider_config.uni_sub_url.clone(),
            None => return Err(eyre!("未找到提供商配置: [providers.{}]", provider)),
        };
        let (interval, strict) = match self.clients.get(&client) {
            Some(ProxyClientConfig::Surge(config)) => (config.interval, config.strict),
            Some(ProxyClientConfig::Clash(config)) => (config.interval, config.strict),
            None => return Err(eyre!("未找到代理客户端配置: [clients.{}]", client)),
        };
        let server = self.server.clone();
        let secret = self.secret.clone();
        let url_builder = UrlBuilder::new(secret, client, provider, server, uni_sub_url, None, interval, strict)?;
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

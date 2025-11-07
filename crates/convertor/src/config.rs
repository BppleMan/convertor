use crate::common::encrypt::encrypt;
use crate::common::once::HOME_CONFIG_DIR;
use crate::config::config_error::ConfigError;
use crate::config::proxy_client::ProxyClient;
use crate::config::redis_config::RedisConfig;
use crate::config::subscription_config::SubscriptionConfig;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::str::FromStr;
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
        let mut builder = config::Config::builder();

        // home 目录
        if let Some(files) = std::env::home_dir().map(|hd| Self::search_dir(hd.join(HOME_CONFIG_DIR))) {
            for file in files {
                builder = builder.add_source(file);
            }
        }

        // 当前工作目录
        if let Some(files) = Self::search_dir(&cwd) {
            for file in files {
                builder = builder.add_source(file);
            }
        }

        // 命令行参数
        if let Some(file) = config_path.map(|p| p.as_ref().to_path_buf()).map(|p| config::File::from(p)) {
            builder = builder.add_source(file);
        }

        // if let Some(path) = config_path {
        //     return Self::from_file(path);
        // }
        // let work_dir = cwd.as_ref().to_path_buf();
        // if !work_dir.is_dir() {
        //     return Err(ConfigError::NotDirectory(work_dir.to_path_buf()));
        // }
        // let work_dir = work_dir.canonicalize().map_err(ConfigError::PathError)?;
        // let convertor_toml = work_dir.join("convertor.toml");
        // debug!("尝试加载配置文件: {}", convertor_toml.display());
        // if convertor_toml.exists() {
        //     Self::from_file(convertor_toml)
        // } else {
        //     Err(ConfigError::NotFound {
        //         cwd: cwd.as_ref().to_path_buf(),
        //         config_name: "convertor.toml".to_string(),
        //     })
        // }
        let config: Self = builder.build()?.try_deserialize()?;
        Ok(config)
    }

    fn search_dir(dir: impl AsRef<Path>) -> Option<Vec<config::File<config::FileSourceFile, config::FileFormat>>> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return None;
        }
        if !dir.is_dir() {
            return None;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return None;
        };
        // 读取目录下所有 convertor.*.{toml,yaml} 文件
        let files = entries
            .flatten()
            .map(|e| {
                (
                    e.path(),
                    e.file_name().display().to_string(),
                    e.path().extension().map(OsStr::to_os_string),
                )
            })
            .filter(|(p, f, e)| {
                p.is_file()
                    && f.starts_with("convertor.")
                    && e.as_ref()
                        .map(|ext| ext == "toml" || ext == "yaml" || ext == "yml")
                        .unwrap_or(false)
            })
            .map(|(p, _, e)| match e.unwrap().to_str().unwrap() {
                "toml" => (p, config::FileFormat::Toml),
                "yaml" | "yml" => (p, config::FileFormat::Yaml),
                _ => unreachable!(),
            })
            .map(|(p, e)| config::File::from(p).format(e))
            .collect::<Vec<_>>();
        Some(files)
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
        let url_builder = UrlBuilder::new(secret, Some(enc_secret), client, server, sub_url, None, interval, strict)?;
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

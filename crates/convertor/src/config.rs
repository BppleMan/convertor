use crate::common::encrypt::encrypt;
use crate::common::once::{HOME_CONFIG_DIR, K8S_CONFIG_DIR};
use crate::config::config_error::ConfigError;
use crate::config::proxy_client::ProxyClient;
use crate::config::redis_config::RedisConfig;
use crate::config::subscription_config::SubscriptionConfig;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, trace};
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

        // 1. Home 目录 (最低优先级)
        if let Some(home_dir) = std::env::home_dir() {
            let config_dir = home_dir.join(HOME_CONFIG_DIR);
            debug!("搜索 home 目录配置: {}", config_dir.display());
            if let Some(files) = Self::search_dir(&config_dir) {
                debug!("在 home 目录找到 {} 个配置文件", files.len());
                for file in &files {
                    trace!("加载配置文件: {:?}", file);
                }
                for file in files {
                    builder = builder.add_source(file);
                }
            }
        }

        // 2. K8s ConfigMap 目录 (中等优先级 - 用于容器部署)
        let k8s_config_dir = std::path::PathBuf::from(K8S_CONFIG_DIR);
        if k8s_config_dir.exists() {
            debug!("搜索 K8s 配置目录: {}", k8s_config_dir.display());
            if let Some(files) = Self::search_dir(&k8s_config_dir) {
                debug!("在 K8s 配置目录找到 {} 个配置文件", files.len());
                for file in &files {
                    trace!("加载配置文件: {:?}", file);
                }
                for file in files {
                    builder = builder.add_source(file);
                }
            }
        }

        // 3. 当前工作目录 (较高优先级)
        let cwd_path = cwd.as_ref();
        debug!("搜索工作目录配置: {}", cwd_path.display());
        if let Some(files) = Self::search_dir(cwd_path) {
            debug!("在工作目录找到 {} 个配置文件", files.len());
            for file in &files {
                trace!("加载配置文件: {:?}", file);
            }
            for file in files {
                builder = builder.add_source(file);
            }
        }

        // 4. 命令行参数 (最高优先级)
        if let Some(file) = config_path.map(|p| p.as_ref().to_path_buf()).map(|p| config::File::from(p)) {
            debug!("加载命令行指定的配置文件");
            builder = builder.add_source(file);
        }

        let config: Self = builder.build()?.try_deserialize()?;
        Ok(config)
    }

    /// 搜索指定目录下的配置文件
    ///
    /// 搜索模式: `convertor.*.{toml,yaml,yml}`
    ///
    /// 加载顺序（同目录内）:
    /// 1. convertor.toml (基础配置)
    /// 2. convertor.yaml / convertor.yml (基础配置)
    /// 3. convertor.{env}.toml (环境特定配置，如 convertor.dev.toml, convertor.prod.toml)
    /// 4. convertor.{env}.yaml (环境特定配置)
    /// 5. convertor.local.toml (本地覆盖配置)
    /// 6. convertor.local.yaml (本地覆盖配置)
    /// 7. 其他 convertor.*.{toml,yaml,yml} (按字母顺序)
    fn search_dir(dir: impl AsRef<Path>) -> Option<Vec<config::File<config::FileSourceFile, config::FileFormat>>> {
        let dir = dir.as_ref();
        if !dir.exists() || !dir.is_dir() {
            return None;
        }

        let Ok(entries) = std::fs::read_dir(dir) else {
            return None;
        };

        // 收集所有匹配的配置文件
        let mut files_map: std::collections::HashMap<String, (std::path::PathBuf, config::FileFormat)> = std::collections::HashMap::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if !path.is_file() || !file_name.starts_with("convertor.") {
                continue;
            }

            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };

            let format = match ext {
                "toml" => config::FileFormat::Toml,
                "yaml" | "yml" => config::FileFormat::Yaml,
                _ => continue,
            };

            trace!("发现配置文件: {} (格式: {:?})", path.display(), format);
            files_map.insert(file_name, (path, format));
        }

        if files_map.is_empty() {
            return None;
        }

        // 按优先级顺序构建文件列表
        let mut files = Vec::new();

        // 1. 基础配置文件
        let base_names = ["convertor.toml", "convertor.yaml", "convertor.yml"];
        for name in &base_names {
            if let Some((path, format)) = files_map.remove(*name) {
                debug!("加载基础配置: {}", path.display());
                files.push(config::File::from(path).format(format));
            }
        }

        // 2. 环境特定配置 (dev, prod, test, staging 等)
        let env_prefixes = ["dev", "prod", "test", "staging"];
        for env in &env_prefixes {
            for ext in &["toml", "yaml", "yml"] {
                let name = format!("convertor.{}.{}", env, ext);
                if let Some((path, format)) = files_map.remove(&name) {
                    debug!("加载环境配置: {}", path.display());
                    files.push(config::File::from(path).format(format));
                }
            }
        }

        // 3. 本地覆盖配置
        let local_names = ["convertor.local.toml", "convertor.local.yaml", "convertor.local.yml"];
        for name in &local_names {
            if let Some((path, format)) = files_map.remove(*name) {
                debug!("加载本地覆盖配置: {}", path.display());
                files.push(config::File::from(path).format(format));
            }
        }

        // 4. 其他配置文件 (按文件名字母顺序)
        let mut remaining: Vec<_> = files_map.into_iter().collect();
        remaining.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, (path, format)) in remaining {
            debug!("加载其他配置: {} ({})", path.display(), name);
            files.push(config::File::from(path).format(format));
        }

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

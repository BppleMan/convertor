use crate::subscription::subscription_config::ServiceConfig;
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorConfig {
    pub secret: String,
    pub server: Url,
    pub service_config: ServiceConfig,
}

impl ConvertorConfig {
    pub fn search(cwd: impl AsRef<Path>, config_path: Option<impl AsRef<Path>>) -> color_eyre::Result<Self> {
        if let Some(path) = config_path {
            return Self::from_file(path);
        }
        let mut current = cwd.as_ref().to_path_buf().canonicalize()?;
        assert!(current.is_dir(), "{}", format!("{} 不是一个目录", current.display()));
        loop {
            let file = current.join("convertor.toml");
            if file.exists() {
                return Self::from_file(file);
            }
            if !current.pop() {
                break;
            }
        }
        let home_dir = std::env::var("HOME")?;
        let convertor_toml = Path::new(&home_dir).join("../../.convertor.dev").join("convertor.toml");
        if convertor_toml.exists() {
            return Self::from_file(convertor_toml);
        }
        Err(eyre!("未找到 convertor.toml 配置文件"))
    }

    pub fn from_file(path: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let path = path.as_ref();
        assert!(path.is_file(), "{}", format!("{} 不是一个文件", path.display()));
        let content = std::fs::read_to_string(path).wrap_err_with(|| eyre!("读取配置文件失败: {}", path.display()))?;
        let config: ConvertorConfig =
            toml::from_str(&content).wrap_err_with(|| format!("解析配置文件失败: {}", path.display()))?;
        Ok(config)
    }

    pub fn server_addr(&self) -> color_eyre::Result<String> {
        self.server
            .host_str()
            .and_then(|host| self.server.port().map(|port| format!("{}:{}", host, port)))
            .ok_or_else(|| eyre!("服务器地址无效"))
    }

    pub fn server_origin(&self) -> color_eyre::Result<String> {
        self.server
            .origin()
            .ascii_serialization()
            .parse()
            .wrap_err("服务器地址无效")
    }
}

impl FromStr for ConvertorConfig {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).wrap_err("解析配置字符串失败")
    }
}

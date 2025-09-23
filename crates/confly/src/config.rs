use color_eyre::Result;
use color_eyre::eyre::eyre;
use convertor::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

mod client_config;

pub use client_config::*;
use convertor::config::proxy_client::ProxyClient;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ConflyConfig {
    #[serde(flatten)]
    pub common: Config,

    #[serde(flatten)]
    pub clients: HashMap<ProxyClient, ClientConfig>,
}

impl ConflyConfig {
    pub fn search(cwd: impl AsRef<Path>, config_path: Option<impl AsRef<Path>>) -> Result<Self> {
        if let Some(path) = config_path {
            return Self::from_file(path);
        }
        let work_dir = cwd.as_ref().to_path_buf();
        if !work_dir.is_dir() {
            return Err(eyre!("工作目录不是一个合法的目录: {}", work_dir.display()));
        }
        let work_dir = work_dir.canonicalize()?;
        let convertor_toml = work_dir.join("convertor.toml");
        debug!("尝试加载配置文件: {}", convertor_toml.display());
        if convertor_toml.exists() {
            Self::from_file(convertor_toml)
        } else {
            Err(eyre!(
                "配置文件 convertor.toml 未找到, 请在当前目录或指定目录下创建配置文件: {}",
                work_dir.display()
            ))
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(eyre!("配置文件不是一个合法的文件: {}", path.display()));
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

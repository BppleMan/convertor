use crate::config::service_config::ServiceConfig;
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Report;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct ConvertorConfig<T: Credential> {
    pub secret: String,
    pub server: Url,
    pub service_config: ServiceConfig<T>,
}

impl<T: Credential> ConvertorConfig<T> {
    pub fn search(cwd: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let mut current = cwd.as_ref().to_path_buf();
        assert!(
            current.is_absolute(),
            "{}",
            format!("{} 不是一个绝对路径", current.display())
        );
        assert!(
            current.is_dir(),
            "{}",
            format!("{} 不是一个目录", current.display())
        );
        loop {
            let file = current.join("../../test-assets/convertor.toml");
            if file.exists() {
                return Self::from_file(file);
            }
            if !current.pop() {
                break;
            }
        }
        Err(eyre!("未找到 convertor.toml 配置文件"))
    }

    pub fn from_file(path: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let path = path.as_ref();
        assert!(
            path.is_file(),
            "{}",
            format!("{} 不是一个文件", path.display())
        );
        let content = std::fs::read_to_string(path)
            .wrap_err_with(|| eyre!("读取配置文件失败: {}", path.display()))?;
        toml::from_str(&content)
            .wrap_err_with(|| format!("解析配置文件失败: {}", path.display()))
    }

    pub fn server_host_with_port(&self) -> color_eyre::Result<String> {
        self.server
            .host_str()
            .and_then(|host| {
                self.server.port().map(|port| format!("{}:{}", host, port))
            })
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

impl<T: Credential> FromStr for ConvertorConfig<T> {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).wrap_err("解析配置字符串失败")
    }
}

pub trait Credential: Debug + Default + Serialize + DeserializeOwned {
    fn get_username(&self) -> &str;

    fn get_password(&self) -> &str;
}

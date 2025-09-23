use color_eyre::Result;
use color_eyre::eyre::eyre;
use convertor::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use tracing::debug;

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

impl ConflyConfig {
    pub fn template() -> Self {
        let common = Config::template();
        let mut clients = HashMap::new();
        clients.insert(ProxyClient::Surge, ClientConfig::surge_template());
        clients.insert(ProxyClient::Clash, ClientConfig::clash_template());
        Self { common, clients }
    }
}

impl Display for ConflyConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).map_err(|_| std::fmt::Error)?)
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    config_dir: PathBuf,
    main_profile: String,
    raw: Option<String>,
    raw_profile: Option<String>,
    rules: Option<String>,
}

impl ClientConfig {
    pub fn surge_template() -> Self {
        Self {
            config_dir: PathBuf::from("/path/to/surge"),
            main_profile: "surge.conf".to_string(),
            ..Default::default()
        }
    }

    pub fn clash_template() -> Self {
        Self {
            config_dir: PathBuf::from("/path/to/mihomo"),
            main_profile: "config.yaml".to_string(),
            ..Default::default()
        }
    }
}

impl ClientConfig {
    pub fn set_config_dir(&mut self, config_dir: impl AsRef<Path>) {
        self.config_dir = config_dir.as_ref().to_path_buf();
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn main_profile_path(&self) -> PathBuf {
        self.config_dir().join(&self.main_profile)
    }

    pub fn raw_path(&self) -> Option<PathBuf> {
        self.raw.as_ref().map(|name| self.config_dir().join(name))
    }

    pub fn raw_profile_path(&self) -> Option<PathBuf> {
        self.raw_profile.as_ref().map(|name| self.config_dir().join(name))
    }

    pub fn rules_path(&self) -> Option<PathBuf> {
        self.rules.as_ref().map(|name| self.config_dir().join(name))
    }
}

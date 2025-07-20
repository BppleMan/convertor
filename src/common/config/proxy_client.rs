use clap::ValueEnum;
use rush_var::expand_env_vars;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
pub enum ProxyClient {
    #[serde(rename = "surge")]
    #[default]
    Surge,
    #[serde(rename = "clash")]
    Clash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyClientConfig {
    pub surge: Option<SurgeConfig>,
    pub clash: Option<ClashConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgeConfig {
    surge_dir: String,
    main_sub_name: String,
    raw_sub_name: Option<String>,
    rules_name: Option<String>,
    sub_logs_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashConfig {
    clash_dir: String,
    main_sub_name: String,
    install_path: Option<String>,
}

impl ProxyClient {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyClient::Surge => "surge",
            ProxyClient::Clash => "clash",
        }
    }
}

impl FromStr for ProxyClient {
    type Err = ParseClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "surge" => Ok(ProxyClient::Surge),
            "clash" => Ok(ProxyClient::Clash),
            _ => Err(ParseClientError(s.to_string())),
        }
    }
}

impl Display for ProxyClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl SurgeConfig {
    pub fn set_surge_dir(&mut self, surge_dir: String) {
        self.surge_dir = surge_dir;
    }

    pub fn surge_dir(&self) -> PathBuf {
        expand_env_vars(&self.surge_dir).into()
    }

    pub fn main_sub_path(&self) -> PathBuf {
        self.surge_dir().join(expand_env_vars(&self.main_sub_name))
    }

    pub fn raw_sub_path(&self) -> Option<PathBuf> {
        self.raw_sub_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }

    pub fn rules_path(&self) -> Option<PathBuf> {
        self.rules_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }

    pub fn sub_logs_path(&self) -> Option<PathBuf> {
        self.sub_logs_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }
}

impl ClashConfig {
    pub fn set_clash_dir(&mut self, clash_dir: String) {
        self.clash_dir = clash_dir;
    }

    pub fn clash_dir(&self) -> PathBuf {
        expand_env_vars(&self.clash_dir).into()
    }

    pub fn main_sub_path(&self) -> PathBuf {
        self.clash_dir().join(expand_env_vars(&self.main_sub_name))
    }

    pub fn install_path(&self) -> Option<PathBuf> {
        self.install_path.as_ref().map(|path| expand_env_vars(path).into())
    }
}

#[derive(Debug, Error)]
#[error("无法解析客户端: {0}")]
pub struct ParseClientError(String);

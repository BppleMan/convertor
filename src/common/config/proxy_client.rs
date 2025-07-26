use clap::ValueEnum;
use dispatch_map::dispatch_pattern;
use rush_var::expand_env_vars;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyClient {
    #[default]
    Surge,
    Clash,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProxyClientConfig {
    Surge(SurgeConfig),
    Clash(ClashConfig),
}

dispatch_pattern! {
    ProxyClient::Surge => ProxyClientConfig::Surge(SurgeConfig),
    ProxyClient::Clash => ProxyClientConfig::Clash(ClashConfig),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SurgeConfig {
    pub interval: u64,
    pub strict: bool,
    surge_dir: String,
    main_profile_name: String,
    raw_profile_name: Option<String>,
    raw_sub_name: Option<String>,
    rules_name: Option<String>,
    sub_logs_name: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ClashConfig {
    clash_dir: String,
    pub interval: u64,
    pub strict: bool,
    main_sub_name: String,
    install_path: Option<String>,
}

impl ProxyClient {
    pub fn clients() -> [ProxyClient; 2] {
        [ProxyClient::Surge, ProxyClient::Clash]
    }

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

impl ProxyClientConfig {
    pub fn interval(&self) -> u64 {
        match self {
            ProxyClientConfig::Surge(config) => config.interval,
            ProxyClientConfig::Clash(config) => config.interval,
        }
    }

    pub fn strict(&self) -> bool {
        match self {
            ProxyClientConfig::Surge(config) => config.strict,
            ProxyClientConfig::Clash(config) => config.strict,
        }
    }
}

impl SurgeConfig {
    pub fn template() -> Self {
        Self {
            interval: 43200,
            strict: true,
            surge_dir: "${ICLOUD}/../iCloud~com~nssurge~inc/Documents/surge".to_string(),
            main_profile_name: "surge.conf".to_string(),
            raw_profile_name: Some("raw.conf".to_string()),
            raw_sub_name: Some("BosLife.conf".to_string()),
            rules_name: Some("rules.dconf".to_string()),
            sub_logs_name: Some("subscription_logs.js".to_string()),
        }
    }

    pub fn set_surge_dir(&mut self, surge_dir: String) {
        self.surge_dir = surge_dir;
    }

    pub fn surge_dir(&self) -> PathBuf {
        expand_env_vars(&self.surge_dir).into()
    }

    pub fn main_profile_path(&self) -> PathBuf {
        self.surge_dir().join(expand_env_vars(&self.main_profile_name))
    }

    pub fn raw_profile_path(&self) -> Option<PathBuf> {
        self.raw_profile_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
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
    pub fn template() -> Self {
        Self {
            clash_dir: "${HOME}/.config/mihomo".to_string(),
            interval: 43200,
            strict: true,
            main_sub_name: "config.yaml".to_string(),
            install_path: Some("/usr/local/bin/mihomo".to_string()),
        }
    }

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

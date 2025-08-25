mod clash_config;
mod surge_config;

pub use clash_config::*;
pub use surge_config::*;

use clap::ValueEnum;
use dispatch_map::dispatch_pattern;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantArray};
use thiserror::Error;

static ENV_VAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\$\{([A-Za-z0-9_]+)}"#).expect("Failed to compile environment variable regex"));

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[derive(ValueEnum, Serialize, Deserialize)]
#[derive(Display, IntoStaticStr, AsRefStr, VariantArray, EnumString)]
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

impl ProxyClientConfig {
    pub fn interval(&self) -> u64 {
        match self {
            ProxyClientConfig::Surge(config) => config.interval(),
            ProxyClientConfig::Clash(config) => config.interval(),
        }
    }

    pub fn strict(&self) -> bool {
        match self {
            ProxyClientConfig::Surge(config) => config.strict(),
            ProxyClientConfig::Clash(config) => config.strict(),
        }
    }

    pub fn surge_template() -> Self {
        ProxyClientConfig::Surge(SurgeConfig::template())
    }

    pub fn clash_template() -> Self {
        ProxyClientConfig::Clash(ClashConfig::template())
    }
}

pub(self) fn expand_env_vars(value: impl AsRef<str>) -> String {
    let value = value.as_ref();
    ENV_VAR_REGEX
        .replace_all(value, |caps: &Captures| {
            let name = &caps[1];
            match std::env::var(name) {
                Ok(value) => value,
                Err(_) => name.to_string(),
            }
        })
        .to_string()
}

#[derive(Debug, Error)]
#[error("无法解析客户端: {0}")]
pub struct ParseClientError(String);

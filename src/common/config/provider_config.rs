mod boslife_config;

pub use boslife_config::*;
use clap::ValueEnum;
use dispatch_map::dispatch_pattern;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantArray};
use thiserror::Error;
use url::Url;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[derive(ValueEnum, Serialize, Deserialize)]
#[derive(Display, IntoStaticStr, AsRefStr, VariantArray, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    BosLife,
}

impl Provider {
    pub fn providers() -> [Self; 1] {
        [Provider::BosLife]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::BosLife => "boslife",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProviderConfig {
    BosLife(BosLifeConfig),
}

dispatch_pattern! {
    Provider::BosLife => ProviderConfig::BosLife(BosLifeConfig),
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CredentialConfig {
    pub username: String,
    pub password: String,
}

impl ProviderConfig {
    pub fn sub_url(&self) -> &Url {
        match self {
            ProviderConfig::BosLife(config) => &config.sub_url,
        }
    }

    pub fn boslife_template() -> Self {
        ProviderConfig::BosLife(BosLifeConfig::template())
    }
}

pub struct ApiConfig {
    pub api: &'static str,
    pub json_path: &'static str,
}

#[derive(Debug, Error)]
#[error("无法解析提供商: {0}")]
pub struct ParseProviderError(String);

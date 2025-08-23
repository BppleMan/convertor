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
pub enum SubProvider {
    #[default]
    BosLife,
}

impl SubProvider {
    pub fn providers() -> [Self; 1] {
        [SubProvider::BosLife]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SubProvider::BosLife => "boslife",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SubProviderConfig {
    BosLife(BosLifeConfig),
}

dispatch_pattern! {
    SubProvider::BosLife => SubProviderConfig::BosLife(BosLifeConfig),
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CredentialConfig {
    pub username: String,
    pub password: String,
}

impl SubProviderConfig {
    pub fn sub_url(&self) -> &Url {
        match self {
            SubProviderConfig::BosLife(config) => &config.sub_url,
        }
    }

    pub fn boslife_template() -> Self {
        SubProviderConfig::BosLife(BosLifeConfig::template())
    }
}

pub struct ApiConfig {
    pub api: &'static str,
    pub json_path: &'static str,
}

#[derive(Debug, Error)]
#[error("无法解析提供商: {0}")]
pub struct ParseProviderError(String);

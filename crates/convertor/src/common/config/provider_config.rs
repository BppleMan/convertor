pub mod boslife_config;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantArray};
use url::Url;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[derive(ValueEnum, Serialize, Deserialize)]
#[derive(Display, IntoStaticStr, AsRefStr, VariantArray, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    BosLife,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: Provider,
    pub sub_url: Url,
    pub api_config: ApiConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: Url,
    pub prefix: String,
    pub headers: Headers,
    pub credential: Credential,
    pub login_api: Api,
    pub get_sub_api: Api,
    pub reset_sub_api: Api,
    pub sub_logs_api: Option<Api>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Api {
    pub path: String,
    pub json_path: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Headers(pub HashMap<String, String>);

impl Hash for Headers {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl AsRef<HashMap<String, String>> for Headers {
    fn as_ref(&self) -> &HashMap<String, String> {
        &self.0
    }
}

impl Deref for Headers {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

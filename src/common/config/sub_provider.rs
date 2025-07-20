use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use url::Url;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SubProvider {
    #[default]
    BosLife,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SubProviderConfig {
    pub provider: SubProvider,
    pub uni_sub_url: Url,
    pub auth_token: String,
    pub cookie: String,
    pub credential: Credential,
    pub api_host: Url,
    pub api_prefix: String,
    pub login_api: Api,
    pub reset_sub_url_api: Api,
    pub get_sub_url_api: Api,
    pub get_sub_logs_api: Api,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Api {
    pub path: String,
    pub json_path: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

impl SubProviderConfig {
    pub fn hash_code(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

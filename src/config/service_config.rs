use crate::config::convertor_config::Credential;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct ServiceConfig<T: Credential> {
    pub base_url: String,
    pub prefix_path: String,
    pub credential: T,
    pub login_api: ConfigApi,
    pub reset_subscription_url: ConfigApi,
    pub get_subscription_url: ConfigApi,
    pub get_subscription_log: ConfigApi,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigApi {
    pub api_path: String,
    pub json_path: String,
}

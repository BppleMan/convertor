use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub base_url: String,
    pub prefix_path: String,
    pub credential: Credential,
    pub login_api: ConfigApi,
    pub reset_subscription_api: ConfigApi,
    pub get_subscription_api: ConfigApi,
    pub get_subscription_logs_api: ConfigApi,
}

impl ServiceConfig {
    pub fn build_login_url(&self) -> color_eyre::Result<Url> {
        let url = Url::parse(&format!(
            "{}{}{}",
            self.base_url, self.prefix_path, self.login_api.api_path
        ))?;
        Ok(url)
    }

    pub fn build_get_subscription_url(&self) -> color_eyre::Result<Url> {
        let url = Url::parse(&format!(
            "{}{}{}",
            self.base_url, self.prefix_path, self.get_subscription_api.api_path
        ))?;
        Ok(url)
    }

    pub fn build_reset_subscription_url(&self) -> color_eyre::Result<Url> {
        let url = Url::parse(&format!(
            "{}{}{}",
            self.base_url, self.prefix_path, self.reset_subscription_api.api_path
        ))?;
        Ok(url)
    }

    pub fn build_get_subscription_logs_url(&self) -> color_eyre::Result<Url> {
        let url = Url::parse(&format!(
            "{}{}{}",
            self.base_url, self.prefix_path, self.get_subscription_logs_api.api_path
        ))?;
        Ok(url)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigApi {
    pub api_path: String,
    pub json_path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

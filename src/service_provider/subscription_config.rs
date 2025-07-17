use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub base_url: Url,
    pub prefix_path: String,
    pub credential: Credential,
    pub login_api: ConfigApi,
    pub reset_sub_api: ConfigApi,
    pub get_sub_api: ConfigApi,
    pub get_sub_logs_api: ConfigApi,
}

impl ServiceConfig {
    pub fn build_login_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .base_url
            .join(&format!("{}{}", self.prefix_path, self.login_api.api_path))?;
        Ok(url)
    }

    pub fn build_get_sub_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .base_url
            .join(&format!("{}{}", self.prefix_path, self.get_sub_api.api_path))?;
        Ok(url)
    }

    pub fn build_reset_sub_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .base_url
            .join(&format!("{}{}", self.prefix_path, self.reset_sub_api.api_path))?;
        Ok(url)
    }

    pub fn build_get_sub_logs_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .base_url
            .join(&format!("{}{}", self.prefix_path, self.get_sub_logs_api.api_path))?;
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

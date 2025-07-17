use crate::client::Client;
use crate::service_provider::ServiceProvider;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service_provider: ServiceProvider,
    pub raw_sub_url: Url,
    pub auth_token: String,
    pub cookie: String,
    pub credential: Credential,
    pub api_host: Url,
    pub api_prefix: String,
    pub login_api: Api,
    pub reset_sub_api: Api,
    pub get_sub_api: Api,
    pub get_sub_logs_api: Api,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Api {
    pub api_path: String,
    pub json_path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

impl ServiceConfig {
    pub fn build_raw_sub_url(&self, client: Client) -> color_eyre::Result<Url> {
        let mut url = self.raw_sub_url.clone();
        // BosLife 的字段是 `flag` 不可改为client
        url.query_pairs_mut().append_pair("flag", client.as_str());
        Ok(url)
    }

    pub fn build_login_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .api_host
            .join(&format!("{}{}", self.api_prefix, self.login_api.api_path))?;
        Ok(url)
    }

    pub fn build_get_sub_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .api_host
            .join(&format!("{}{}", self.api_prefix, self.get_sub_api.api_path))?;
        Ok(url)
    }

    pub fn build_reset_sub_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .api_host
            .join(&format!("{}{}", self.api_prefix, self.reset_sub_api.api_path))?;
        Ok(url)
    }

    pub fn build_get_sub_logs_url(&self) -> color_eyre::Result<Url> {
        let url = self
            .api_host
            .join(&format!("{}{}", self.api_prefix, self.get_sub_logs_api.api_path))?;
        Ok(url)
    }
}

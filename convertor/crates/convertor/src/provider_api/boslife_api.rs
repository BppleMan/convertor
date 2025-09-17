use crate::common::cache::Cache;
use crate::config::client_config::ProxyClient;
use crate::config::provider_config::{ApiConfig, ProviderConfig};
use crate::provider_api::provider_api_trait::ProviderApiTrait;
use color_eyre::eyre::{eyre, Context};
use redis::aio::ConnectionManager;
use reqwest::Client as ReqwestClient;
use reqwest::{Method, Request, Url};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone)]
pub struct BosLifeApi {
    pub config: ProviderConfig,
    pub client: ReqwestClient,
    pub cached_auth_token: Cache<String, String>,
    pub cached_sub_profile: Cache<String, String>,
    pub cached_sub_url: Cache<String, String>,
    pub cached_sub_logs: Cache<String, BosLifeLogs>,
}

impl BosLifeApi {
    pub fn new(config: ProviderConfig, redis: Option<ConnectionManager>) -> Self {
        let client = ReqwestClient::builder()
            .cookie_store(true)
            .use_rustls_tls()
            .build()
            .expect("Failed to create Reqwest client");
        let duration = std::time::Duration::from_secs(60 * 60); // 1 hour
        let cached_sub_profile = Cache::new(redis.clone(), 10, duration);
        let cached_auth_token = Cache::new(redis.clone(), 10, duration);
        let cached_sub_url = Cache::new(redis.clone(), 10, duration);
        let cached_sub_logs = Cache::new(redis.clone(), 10, duration);
        Self {
            config,
            client,
            cached_auth_token,
            cached_sub_profile,
            cached_sub_url,
            cached_sub_logs,
        }
    }
}

impl ProviderApiTrait for BosLifeApi {
    fn api_config(&self) -> &ApiConfig {
        &self.config.api_config
    }

    fn build_raw_url(&self, client: ProxyClient) -> Url {
        let mut sub_url = self.config.sub_url.clone();
        sub_url.query_pairs_mut().append_pair("flag", client.into());
        sub_url
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        let builder = self.client.request(Method::POST, self.api_config().login_url()).form(&[
            ("email", self.api_config().credential.username.clone()),
            ("password", self.api_config().credential.password.clone()),
        ]);
        builder.build().wrap_err(eyre!("构建登录请求失败"))
    }

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let builder = self
            .client
            .request(Method::GET, self.api_config().get_sub_url())
            .header("Authorization", auth_token.as_ref());
        Ok(builder.build()?)
    }

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let builder = self
            .client
            .request(Method::POST, self.api_config().reset_sub_url())
            .header("Authorization", auth_token.as_ref());
        Ok(builder.build()?)
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.api_config().sub_logs_url().ok_or(eyre!("订阅日志接口未配置"))?;
        let builder = self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref());
        Ok(builder.build()?)
    }

    fn cached_profile(&self) -> &Cache<String, String> {
        &self.cached_sub_profile
    }

    fn cached_auth_token(&self) -> &Cache<String, String> {
        &self.cached_auth_token
    }

    fn cached_sub_url(&self) -> &Cache<String, String> {
        &self.cached_sub_url
    }

    fn cached_sub_logs(&self) -> &Cache<String, BosLifeLogs> {
        &self.cached_sub_logs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BosLifeLog {
    pub user_id: u64,
    pub ip: String,
    pub location: String,
    pub isp: String,
    pub host: String,
    pub ua: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BosLifeLogs(pub Vec<BosLifeLog>);

impl Display for BosLifeLogs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = serde_json::to_string_pretty(self).expect("JSON serialization failed");
        write!(f, "{string}")
    }
}

impl From<String> for BosLifeLogs {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).expect("JSON deserialization failed")
    }
}

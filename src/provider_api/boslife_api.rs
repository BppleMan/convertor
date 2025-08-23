use crate::common::cache::Cache;
use crate::common::config::provider_config::{ApiConfig, BosLifeConfig};
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::config::request_config::RequestConfig;
use crate::common::ext::NonEmptyOptStr;
use crate::provider_api::provider_api_trait::ProviderApiTrait;
use redis::aio::ConnectionManager;
use reqwest::Client as ReqwestClient;
use reqwest::{Method, Request, Url};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone)]
pub struct BosLifeApi {
    pub config: BosLifeConfig,
    pub client: ReqwestClient,
    pub cached_auth_token: Cache<String, String>,
    pub cached_sub_profile: Cache<String, String>,
    pub cached_sub_url: Cache<String, String>,
    pub cached_sub_logs: Cache<String, BosLifeLogs>,
}

impl BosLifeApi {
    pub fn new(config: BosLifeConfig, redis: Option<ConnectionManager>) -> Self {
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
    fn request_config(&self) -> Option<&RequestConfig> {
        self.config.request.as_ref()
    }

    fn login_url_api(&self) -> ApiConfig {
        self.config.login_url_api()
    }

    fn get_sub_url_api(&self) -> ApiConfig {
        self.config.get_sub_url_api()
    }

    fn reset_sub_url_api(&self) -> ApiConfig {
        self.config.reset_sub_url_api()
    }

    fn get_sub_logs_url_api(&self) -> ApiConfig {
        self.config.get_sub_logs_url_api()
    }

    fn build_raw_sub_url(&self, client: ProxyClient) -> Url {
        self.config.build_raw_sub_url(client)
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        let url = self.login_url_api().api;
        let mut builder = self.client.request(Method::POST, url).form(&[
            ("email", self.config.credential.username.clone()),
            ("password", self.config.credential.password.clone()),
        ]);
        if let Some(config) = self.config.request.as_ref() {
            if let Some(user_agent) = config.user_agent.filter_non_empty() {
                builder = builder.header("User-Agent", user_agent);
            }
            for (k, v) in &config.headers {
                builder = builder.header(k, v);
            }
        }
        Ok(builder.build()?)
    }

    fn get_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.get_sub_url_api().api;
        let mut builder = self.client.request(Method::GET, url);
        if let Some(config) = self.config.request.as_ref() {
            builder = config.patch_request(builder, Some(auth_token));
        }
        Ok(builder.build()?)
    }

    fn reset_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.reset_sub_url_api().api;
        let mut builder = self.client.request(Method::POST, url);
        if let Some(config) = self.config.request.as_ref() {
            builder = config.patch_request(builder, Some(auth_token));
        }
        Ok(builder.build()?)
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.get_sub_logs_url_api().api;
        let mut builder = self.client.request(Method::GET, url);
        if let Some(config) = self.config.request.as_ref() {
            builder = config.patch_request(builder, Some(auth_token));
        }
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
        write!(f, "{}", string)
    }
}

impl From<String> for BosLifeLogs {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).expect("JSON deserialization failed")
    }
}

use crate::api::boslife_log::BosLifeLogs;
use crate::api::provider::ProviderApi;
use crate::common::cache::Cache;
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::request::RequestConfig;
use crate::common::config::sub_provider::{ApiConfig, BosLifeConfig};
use moka::future::Cache as MokaCache;
use redis::aio::ConnectionManager;
use reqwest::Client as ReqwestClient;
use reqwest::{Method, Request, Url};

#[derive(Clone)]
pub struct BosLifeApi {
    pub config: BosLifeConfig,
    pub client: ReqwestClient,
    pub redis: ConnectionManager,
    pub cached_auth_token: MokaCache<String, String>,
    pub cached_sub_profile: Cache<Url, String>,
    pub cached_sub_url: Cache<Url, String>,
    pub cached_sub_logs: Cache<Url, BosLifeLogs>,
}

impl BosLifeApi {
    pub fn new(redis: ConnectionManager, config: BosLifeConfig) -> Self {
        let client = ReqwestClient::builder()
            .cookie_store(true)
            .use_rustls_tls()
            .build()
            .expect("Failed to create Reqwest client");
        let duration = std::time::Duration::from_secs(60 * 60); // 1 hour
        let cached_sub_profile = Cache::new(redis.clone(), 10, duration);
        let cached_auth_token = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration) // 10 minutes
            .build();
        let cached_sub_url = Cache::new(redis.clone(), 10, duration);
        let cached_sub_logs = Cache::new(redis.clone(), 10, duration);
        Self {
            config,
            client,
            redis,
            cached_auth_token,
            cached_sub_profile,
            cached_sub_url,
            cached_sub_logs,
        }
    }
}

impl ProviderApi for BosLifeApi {
    fn common_request_config(&self) -> Option<&RequestConfig> {
        self.config.request.as_ref()
    }

    fn api_host(&self) -> &Url {
        &self.config.api_host
    }

    fn login_api(&self) -> &ApiConfig {
        &self.config.login_api
    }

    fn get_sub_url_api(&self) -> &ApiConfig {
        &self.config.get_sub_url_api
    }

    fn reset_sub_url_api(&self) -> &ApiConfig {
        &self.config.reset_sub_url_api
    }

    fn get_sub_logs_api(&self) -> &ApiConfig {
        &self.config.get_sub_logs_api
    }

    fn build_raw_sub_url(&self, client: ProxyClient) -> Url {
        self.config.build_raw_sub_url(client)
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        let url = self.config.build_login_url();
        let mut builder = self.client.request(Method::POST, url).form(&[
            ("email", self.config.credential.username.clone()),
            ("password", self.config.credential.password.clone()),
        ]);
        if let Some(config) = self.config.request.as_ref() {
            if let Some(user_agent) = &config.user_agent {
                builder = builder.header("User-Agent", user_agent);
            }
            for (k, v) in &config.headers {
                builder = builder.header(k, v);
            }
        }
        Ok(builder.build()?)
    }

    fn get_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_url();
        let mut builder = self.client.request(Method::GET, url);
        if let Some(config) = self.config.request.as_ref() {
            builder = config.patch_request(builder, Some(auth_token));
        }
        Ok(builder.build()?)
    }

    fn reset_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_reset_sub_url();
        let mut builder = self.client.request(Method::POST, url);
        if let Some(config) = self.config.request.as_ref() {
            builder = config.patch_request(builder, Some(auth_token));
        }
        Ok(builder.build()?)
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_logs_url();
        let mut builder = self.client.request(Method::GET, url);
        if let Some(config) = self.config.request.as_ref() {
            builder = config.patch_request(builder, Some(auth_token));
        }
        Ok(builder.build()?)
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        &self.cached_auth_token
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        &self.cached_sub_profile
    }

    fn cached_sub_url(&self) -> &Cache<Url, String> {
        &self.cached_sub_url
    }

    fn cached_sub_logs(&self) -> &Cache<Url, BosLifeLogs> {
        &self.cached_sub_logs
    }
}

use crate::api::boslife_sub_log::BosLifeSubLogs;
use crate::api::sub_provider::SubProviderApi;
use crate::common::cache::Cache;
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::request::RequestConfig;
use crate::common::config::sub_provider::{ApiConfig, BosLifeConfig};
use moka::future::Cache as MokaCache;
use reqwest::Client as ReqwestClient;
use reqwest::{Method, Request, Url};
use std::path::Path;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct BosLifeApi {
    pub config: BosLifeConfig,
    pub client: ReqwestClient,
    pub cached_auth_token: MokaCache<String, String>,
    pub cached_sub_profile: Cache<Url, String>,
    pub cached_sub_url: Cache<Url, String>,
    pub cached_sub_logs: Cache<Url, BosLifeSubLogs>,
}

impl BosLifeApi {
    pub fn new(base_dir: impl AsRef<Path>, config: BosLifeConfig) -> Self {
        let client = ReqwestClient::builder()
            .cookie_store(true)
            .use_rustls_tls()
            .build()
            .expect("Failed to create Reqwest client");
        let duration = std::time::Duration::from_secs(60 * 60); // 1 hour
        let cached_file = Cache::new(10, base_dir.as_ref(), duration);
        let cached_string = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration) // 10 minutes
            .build();
        let cached_uni_sub_url = Cache::new(10, base_dir.as_ref(), duration);
        let cached_sub_logs = Cache::new(10, base_dir.as_ref(), duration);
        Self {
            config,
            client,
            cached_auth_token: cached_string,
            cached_sub_profile: cached_file,
            cached_sub_url: cached_uni_sub_url,
            cached_sub_logs,
        }
    }
}

impl SubProviderApi for BosLifeApi {
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
        let request = self
            .client
            .request(Method::POST, url)
            .form(&[
                ("email", self.config.credential.username.clone()),
                ("password", self.config.credential.password.clone()),
            ])
            .header("User-Agent", USER_AGENT)
            .build()?;
        Ok(request)
    }

    fn get_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_url();
        let mut request_builder = self.client.request(Method::GET, url);
        if let Some(common_request) = self.config.request.as_ref() {
            request_builder = common_request.patch_request(request_builder, Some(auth_token));
        }
        Ok(request_builder.build()?)
    }

    fn reset_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_reset_sub_url();
        let mut request_builder = self.client.request(Method::POST, url);
        if let Some(common_request) = self.config.request.as_ref() {
            request_builder = common_request.patch_request(request_builder, Some(auth_token));
        }
        Ok(request_builder.build()?)
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_logs_url();
        let mut request_builder = self.client.request(Method::GET, url);
        if let Some(common_request) = self.config.request.as_ref() {
            request_builder = common_request.patch_request(request_builder, Some(auth_token));
        }
        Ok(request_builder.build()?)
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

    fn cached_sub_logs(&self) -> &Cache<Url, BosLifeSubLogs> {
        &self.cached_sub_logs
    }
}

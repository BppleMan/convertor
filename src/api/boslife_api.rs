use crate::api::boslife_sub_log::BosLifeSubLogs;
use crate::api::sub_provider_api::SubProviderApi;
use crate::common::cache::Cache;
use crate::common::config::sub_provider::SubProviderConfig;
use moka::future::Cache as MokaCache;
use reqwest::Client as ReqwestClient;
use reqwest::{Method, Request, Url};
use std::path::Path;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct BosLifeApi {
    pub config: SubProviderConfig,
    pub client: ReqwestClient,
    pub cached_auth_token: MokaCache<String, String>,
    pub cached_sub_profile: Cache<Url, String>,
    pub cached_uni_sub_url: Cache<Url, String>,
    pub cached_sub_logs: Cache<Url, BosLifeSubLogs>,
}

impl BosLifeApi {
    pub fn new(base_dir: impl AsRef<Path>, config: SubProviderConfig) -> Self {
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
            cached_uni_sub_url,
            cached_sub_logs,
        }
    }
}

impl SubProviderApi for BosLifeApi {
    fn config(&self) -> &SubProviderConfig {
        &self.config
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        let url = self.config.build_login_url()?;
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

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_url()?;
        let request = self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref())
            .header("User-Agent", USER_AGENT)
            .build()?;
        Ok(request)
    }

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_reset_sub_url()?;
        let request = self
            .client
            .request(Method::POST, url)
            .header("Authorization", auth_token.as_ref())
            .header("User-Agent", USER_AGENT)
            .build()?;
        Ok(request)
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_logs_url()?;
        let request = self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref())
            .header("Cookie", &self.config.cookie)
            .header("User-Agent", USER_AGENT)
            .build()?;
        Ok(request)
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        &self.cached_auth_token
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        &self.cached_sub_profile
    }

    fn cached_raw_sub_url(&self) -> &Cache<Url, String> {
        &self.cached_uni_sub_url
    }

    fn cached_sub_logs(&self) -> &Cache<Url, BosLifeSubLogs> {
        &self.cached_sub_logs
    }
}

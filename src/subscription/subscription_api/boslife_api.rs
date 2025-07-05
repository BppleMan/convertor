use crate::cache::Cache;
use crate::client::Client;
use crate::subscription::subscription_api::ServiceApi;
use crate::subscription::subscription_config::ServiceConfig;
use crate::subscription::subscription_log::SubscriptionLog;
use moka::future::Cache as MokaCache;
use reqwest::{Method, Request, Url};
use std::path::Path;

#[derive(Clone)]
pub struct BosLifeApi {
    pub config: ServiceConfig,
    pub client: reqwest::Client,
    pub cached_auth_token: MokaCache<String, String>,
    pub cached_sub_profile: Cache<Url, String>,
    pub cached_raw_sub_url: Cache<Url, String>,
    pub cached_sub_logs: MokaCache<String, Vec<SubscriptionLog>>,
}

impl BosLifeApi {
    pub fn new(base_dir: impl AsRef<Path>, client: reqwest::Client, config: ServiceConfig) -> Self {
        let duration = std::time::Duration::from_secs(60 * 10); // 10 minutes
        let cached_file = Cache::new(10, base_dir.as_ref(), duration);
        let cached_string = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration) // 10 minutes
            .build();
        let cached_raw_subscription_url = Cache::new(10, base_dir.as_ref(), duration);
        let cached_subscription_logs = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration) // 10 minutes
            .build();
        Self {
            config,
            client,
            cached_auth_token: cached_string,
            cached_sub_profile: cached_file,
            cached_raw_sub_url: cached_raw_subscription_url,
            cached_sub_logs: cached_subscription_logs,
        }
    }

    /// sub_url 是订阅地址并且应该指定客户端
    pub async fn get_raw_profile(&self, sub_url: Url, client: Client) -> color_eyre::Result<String> {
        ServiceApi::get_raw_profile(self, sub_url, client).await
    }

    pub async fn get_raw_sub_url(&self, base_url: Url, client: Client) -> color_eyre::Result<Url> {
        ServiceApi::get_raw_sub_url(self, base_url, client).await
    }

    pub async fn reset_raw_sub_url(&self, base_url: Url) -> color_eyre::Result<Url> {
        ServiceApi::reset_raw_sub_url(self, base_url).await
    }

    pub async fn get_sub_logs(&self, base_url: Url) -> color_eyre::Result<Vec<SubscriptionLog>> {
        ServiceApi::get_sub_logs(self, base_url).await
    }
}

impl ServiceApi for BosLifeApi {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        let url = self.config.build_login_url()?;
        Ok(self
            .client
            .request(Method::POST, url)
            .form(&[
                ("email", self.config.credential.username.clone()),
                ("password", self.config.credential.password.clone()),
            ])
            .build()?)
    }

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_url()?;
        Ok(self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref())
            .build()?)
    }

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_reset_sub_url()?;
        Ok(self
            .client
            .request(Method::POST, url)
            .header("Authorization", auth_token.as_ref())
            .build()?)
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_sub_logs_url()?;
        Ok(self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref())
            .build()?)
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        &self.cached_auth_token
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        &self.cached_sub_profile
    }

    fn cached_raw_sub_url(&self) -> &Cache<Url, String> {
        &self.cached_raw_sub_url
    }

    fn cached_sub_logs(&self) -> &MokaCache<String, Vec<SubscriptionLog>> {
        &self.cached_sub_logs
    }
}

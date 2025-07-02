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
    pub cached_file: Cache<Url, String>,
    pub cached_string: MokaCache<String, String>,
    pub cached_raw_subscription_url: MokaCache<String, Url>,
    pub cached_subscription_logs: MokaCache<String, Vec<SubscriptionLog>>,
}

impl BosLifeApi {
    pub fn new(base_dir: impl AsRef<Path>, client: reqwest::Client, config: ServiceConfig) -> Self {
        let duration = std::time::Duration::from_secs(60 * 10); // 10 minutes
        let cached_file = Cache::new(10, base_dir, duration.clone());
        let cached_string = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration.clone()) // 10 minutes
            .build();
        let cached_raw_subscription_url = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration.clone()) // 10 minutes
            .build();
        let cached_subscription_logs = MokaCache::builder()
            .max_capacity(10)
            .time_to_live(duration.clone()) // 10 minutes
            .build();
        Self {
            config,
            client,
            cached_file,
            cached_string,
            cached_raw_subscription_url,
            cached_subscription_logs,
        }
    }

    pub async fn get_raw_profile(&self, url: Url, client: Client) -> color_eyre::Result<String> {
        ServiceApi::get_raw_profile(self, url, client).await
    }

    pub async fn get_raw_subscription_url(&self) -> color_eyre::Result<Url> {
        ServiceApi::get_raw_subscription_url(self).await
    }

    pub async fn reset_raw_subscription_url(&self) -> color_eyre::Result<Url> {
        ServiceApi::reset_raw_subscription_url(self).await
    }

    pub async fn get_subscription_logs(&self) -> color_eyre::Result<Vec<SubscriptionLog>> {
        ServiceApi::get_subscription_logs(self).await
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

    fn get_subscription_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_subscription_url()?;
        Ok(self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref())
            .build()?)
    }

    fn reset_subscription_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_reset_subscription_url()?;
        Ok(self
            .client
            .request(Method::POST, url)
            .header("Authorization", auth_token.as_ref())
            .build()?)
    }

    fn get_subscription_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        let url = self.config.build_get_subscription_logs_url()?;
        Ok(self
            .client
            .request(Method::GET, url)
            .header("Authorization", auth_token.as_ref())
            .build()?)
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        &self.cached_string
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        &self.cached_file
    }

    fn cached_raw_subscription_url(&self) -> &MokaCache<String, Url> {
        &self.cached_raw_subscription_url
    }

    fn cached_subscription_logs(&self) -> &MokaCache<String, Vec<SubscriptionLog>> {
        &self.cached_subscription_logs
    }
}

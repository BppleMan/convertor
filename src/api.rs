use crate::api::boslife::BosLifeApi;
use crate::api::boslife_log::BosLifeLogs;
use crate::api::provider::ProviderApi;
use crate::common::cache::Cache;
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::request::RequestConfig;
use crate::common::config::sub_provider::{ApiConfig, SubProvider, SubProviderConfig};
use dispatch_map::DispatchMap;
use moka::future::Cache as MokaCache;
use redis::aio::ConnectionManager;
use reqwest::{Client as ReqwestClient, Request};
use std::collections::HashMap;
use url::Url;

mod boslife;
pub mod boslife_log;
pub mod provider;

#[derive(Clone)]
pub enum SubProviderWrapper {
    BosLife(BosLifeApi),
}

impl SubProviderWrapper {
    pub fn create_api(
        providers: DispatchMap<SubProvider, SubProviderConfig>,
        redis: ConnectionManager,
    ) -> HashMap<SubProvider, SubProviderWrapper> {
        providers
            .into_iter()
            .map(|(provider, config)| match (provider, config) {
                (SubProvider::BosLife, SubProviderConfig::BosLife(config)) => (
                    SubProvider::BosLife,
                    SubProviderWrapper::BosLife(BosLifeApi::new(redis.clone(), config)),
                ),
            })
            .collect()
    }

    pub fn set_uni_sub_url(&mut self, url: Url) {
        match self {
            SubProviderWrapper::BosLife(api) => api.config.uni_sub_url = url,
        }
    }

    pub fn client(&self) -> &ReqwestClient {
        match self {
            SubProviderWrapper::BosLife(api) => api.client(),
        }
    }

    /// sub_url 是订阅地址并且应该指定客户端
    pub async fn get_raw_profile(&self, client: ProxyClient) -> color_eyre::Result<String> {
        ProviderApi::get_raw_profile(self, client).await
    }

    pub async fn get_sub_url(&self) -> color_eyre::Result<Url> {
        ProviderApi::get_uni_sub_url(self).await
    }

    pub async fn reset_sub_url(&self) -> color_eyre::Result<Url> {
        ProviderApi::reset_uni_sub_url(self).await
    }

    pub async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeLogs> {
        ProviderApi::get_sub_logs(self).await
    }
}

impl ProviderApi for SubProviderWrapper {
    fn common_request_config(&self) -> Option<&RequestConfig> {
        match self {
            SubProviderWrapper::BosLife(api) => api.common_request_config(),
        }
    }

    fn api_host(&self) -> &Url {
        match self {
            SubProviderWrapper::BosLife(api) => api.api_host(),
        }
    }

    fn login_api(&self) -> &ApiConfig {
        match self {
            SubProviderWrapper::BosLife(api) => api.login_api(),
        }
    }

    fn get_sub_url_api(&self) -> &ApiConfig {
        match self {
            SubProviderWrapper::BosLife(api) => api.get_sub_url_api(),
        }
    }

    fn reset_sub_url_api(&self) -> &ApiConfig {
        match self {
            SubProviderWrapper::BosLife(api) => api.reset_sub_url_api(),
        }
    }

    fn get_sub_logs_api(&self) -> &ApiConfig {
        match self {
            SubProviderWrapper::BosLife(api) => api.get_sub_logs_api(),
        }
    }

    fn build_raw_sub_url(&self, client: ProxyClient) -> Url {
        match self {
            SubProviderWrapper::BosLife(api) => api.build_raw_sub_url(client),
        }
    }

    fn client(&self) -> &ReqwestClient {
        match self {
            SubProviderWrapper::BosLife(api) => api.client(),
        }
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        match self {
            SubProviderWrapper::BosLife(api) => api.login_request(),
        }
    }

    fn get_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            SubProviderWrapper::BosLife(api) => api.get_sub_url_request(auth_token),
        }
    }

    fn reset_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            SubProviderWrapper::BosLife(api) => api.reset_sub_url_request(auth_token),
        }
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            SubProviderWrapper::BosLife(api) => api.get_sub_logs_request(auth_token),
        }
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        match self {
            SubProviderWrapper::BosLife(api) => api.cached_auth_token(),
        }
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        match self {
            SubProviderWrapper::BosLife(api) => api.cached_profile(),
        }
    }

    fn cached_sub_url(&self) -> &Cache<Url, String> {
        match self {
            SubProviderWrapper::BosLife(api) => api.cached_sub_url(),
        }
    }

    fn cached_sub_logs(&self) -> &Cache<Url, BosLifeLogs> {
        match self {
            SubProviderWrapper::BosLife(api) => api.cached_sub_logs(),
        }
    }
}

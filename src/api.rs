use crate::api::boslife_api::BosLifeApi;
use crate::api::boslife_sub_log::BosLifeSubLogs;
use crate::api::common::ServiceApiCommon;
use crate::common::cache::Cache;
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::sub_provider::{SubProvider, SubProviderConfig};
use moka::future::Cache as MokaCache;
use reqwest::{Client as ReqwestClient, Request};
use std::path::Path;
use url::Url;

mod boslife_api;
pub mod boslife_sub_log;
pub mod common;

pub enum SubProviderApi {
    BosLife(BosLifeApi),
}

impl SubProviderApi {
    pub fn get_service_provider_api(config: SubProviderConfig, base_dir: impl AsRef<Path>) -> SubProviderApi {
        match config.provider {
            SubProvider::BosLife => SubProviderApi::BosLife(BosLifeApi::new(base_dir, config)),
        }
    }

    pub fn client(&self) -> &ReqwestClient {
        match self {
            SubProviderApi::BosLife(api) => api.client(),
        }
    }

    /// sub_url 是订阅地址并且应该指定客户端
    pub async fn get_raw_profile(&self, client: ProxyClient) -> color_eyre::Result<String> {
        ServiceApiCommon::get_raw_profile(self, client).await
    }

    pub async fn get_raw_sub_url(&self) -> color_eyre::Result<Url> {
        ServiceApiCommon::get_raw_sub_url(self).await
    }

    pub async fn reset_raw_sub_url(&self) -> color_eyre::Result<Url> {
        ServiceApiCommon::reset_raw_sub_url(self).await
    }

    pub async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeSubLogs> {
        ServiceApiCommon::get_sub_logs(self).await
    }
}

impl ServiceApiCommon for SubProviderApi {
    fn config(&self) -> &SubProviderConfig {
        match self {
            SubProviderApi::BosLife(api) => api.config(),
        }
    }

    fn client(&self) -> &ReqwestClient {
        match self {
            SubProviderApi::BosLife(api) => api.client(),
        }
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        match self {
            SubProviderApi::BosLife(api) => api.login_request(),
        }
    }

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            SubProviderApi::BosLife(api) => api.get_sub_request(auth_token),
        }
    }

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            SubProviderApi::BosLife(api) => api.reset_sub_request(auth_token),
        }
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            SubProviderApi::BosLife(api) => api.get_sub_logs_request(auth_token),
        }
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        match self {
            SubProviderApi::BosLife(api) => api.cached_auth_token(),
        }
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        match self {
            SubProviderApi::BosLife(api) => api.cached_profile(),
        }
    }

    fn cached_raw_sub_url(&self) -> &Cache<Url, String> {
        match self {
            SubProviderApi::BosLife(api) => api.cached_raw_sub_url(),
        }
    }

    fn cached_sub_logs(&self) -> &Cache<Url, BosLifeSubLogs> {
        match self {
            SubProviderApi::BosLife(api) => api.cached_sub_logs(),
        }
    }
}

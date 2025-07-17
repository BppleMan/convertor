use crate::cache::Cache;
use crate::client::Client;
use crate::service_provider::ServiceProvider;
use crate::service_provider::api::boslife_api::BosLifeApi;
use crate::service_provider::api::common::ServiceApiCommon;
use crate::service_provider::api::subscription_log::SubscriptionLogs;
use crate::service_provider::config::ServiceConfig;
use moka::future::Cache as MokaCache;
use reqwest::{Client as ReqwestClient, Request};
use std::path::Path;
use url::Url;

mod boslife_api;
pub mod subscription_log;
pub mod common;

pub enum ServiceApi {
    BosLife(BosLifeApi),
}

impl ServiceApi {
    pub fn get_service_provider_api(
        config: ServiceConfig,
        base_dir: impl AsRef<Path>,
        client: ReqwestClient,
    ) -> ServiceApi {
        match config.service_provider {
            ServiceProvider::BosLife => ServiceApi::BosLife(BosLifeApi::new(base_dir, client, config)),
        }
    }

    /// sub_url 是订阅地址并且应该指定客户端
    pub async fn get_raw_profile(&self, client: Client) -> color_eyre::Result<String> {
        ServiceApiCommon::get_raw_profile(self, client).await
    }

    pub async fn get_raw_sub_url(&self) -> color_eyre::Result<Url> {
        ServiceApiCommon::get_raw_sub_url(self).await
    }

    pub async fn reset_raw_sub_url(&self) -> color_eyre::Result<Url> {
        ServiceApiCommon::reset_raw_sub_url(self).await
    }

    pub async fn get_sub_logs(&self) -> color_eyre::Result<SubscriptionLogs> {
        ServiceApiCommon::get_sub_logs(self).await
    }
}

impl ServiceApiCommon for ServiceApi {
    fn config(&self) -> &ServiceConfig {
        match self {
            ServiceApi::BosLife(api) => api.config(),
        }
    }

    fn client(&self) -> &ReqwestClient {
        match self {
            ServiceApi::BosLife(api) => api.client(),
        }
    }

    fn login_request(&self) -> color_eyre::Result<Request> {
        match self {
            ServiceApi::BosLife(api) => api.login_request(),
        }
    }

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            ServiceApi::BosLife(api) => api.get_sub_request(auth_token),
        }
    }

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            ServiceApi::BosLife(api) => api.reset_sub_request(auth_token),
        }
    }

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request> {
        match self {
            ServiceApi::BosLife(api) => api.get_sub_logs_request(auth_token),
        }
    }

    fn cached_auth_token(&self) -> &MokaCache<String, String> {
        match self {
            ServiceApi::BosLife(api) => api.cached_auth_token(),
        }
    }

    fn cached_profile(&self) -> &Cache<Url, String> {
        match self {
            ServiceApi::BosLife(api) => api.cached_profile(),
        }
    }

    fn cached_raw_sub_url(&self) -> &Cache<Url, String> {
        match self {
            ServiceApi::BosLife(api) => api.cached_raw_sub_url(),
        }
    }

    fn cached_sub_logs(&self) -> &Cache<Url, SubscriptionLogs> {
        match self {
            ServiceApi::BosLife(api) => api.cached_sub_logs(),
        }
    }
}

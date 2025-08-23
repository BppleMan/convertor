use crate::common::config::provider::{SubProvider, SubProviderConfig};
use crate::common::config::proxy_client::ProxyClient;
use crate::provider_api::boslife_api::BosLifeApi;
use crate::provider_api::boslife_log::BosLifeLogs;
use crate::provider_api::provider_api_trait::ProviderApiTrait;
use axum_extra::headers::UserAgent;
use dispatch_map::DispatchMap;
use redis::aio::ConnectionManager;
use reqwest::Client as ReqwestClient;
use std::collections::HashMap;
use url::Url;

mod boslife_api;
pub mod boslife_log;
pub mod provider_api_trait;

#[derive(Clone)]
pub enum ProviderApi {
    BosLife(BosLifeApi),
}

impl ProviderApi {
    pub fn create_api(
        providers: DispatchMap<SubProvider, SubProviderConfig>,
        redis: ConnectionManager,
    ) -> HashMap<SubProvider, ProviderApi> {
        providers
            .into_iter()
            .map(|(provider, config)| match (provider, config) {
                (SubProvider::BosLife, SubProviderConfig::BosLife(config)) => (
                    SubProvider::BosLife,
                    ProviderApi::BosLife(BosLifeApi::new(config, Some(redis.clone()))),
                ),
            })
            .collect()
    }

    pub fn create_api_no_redis(
        providers: DispatchMap<SubProvider, SubProviderConfig>,
    ) -> HashMap<SubProvider, ProviderApi> {
        providers
            .into_iter()
            .map(|(provider, config)| match (provider, config) {
                (SubProvider::BosLife, SubProviderConfig::BosLife(config)) => (
                    SubProvider::BosLife,
                    ProviderApi::BosLife(BosLifeApi::new(config, None)),
                ),
            })
            .collect()
    }

    pub fn set_sub_url(&mut self, url: Url) {
        match self {
            ProviderApi::BosLife(api) => api.config.sub_url = url,
        }
    }

    pub fn client(&self) -> &ReqwestClient {
        match self {
            ProviderApi::BosLife(api) => api.client(),
        }
    }

    // sub_url 是订阅地址并且应该指定客户端
    pub async fn get_raw_profile(&self, client: ProxyClient, user_agent: UserAgent) -> color_eyre::Result<String> {
        match self {
            ProviderApi::BosLife(api) => api.get_raw_profile(client, user_agent).await,
        }
    }

    pub async fn get_sub_url(&self) -> color_eyre::Result<Url> {
        match self {
            ProviderApi::BosLife(api) => api.get_sub_url().await,
        }
    }

    pub async fn reset_sub_url(&self) -> color_eyre::Result<Url> {
        match self {
            ProviderApi::BosLife(api) => api.reset_sub_url().await,
        }
    }

    pub async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeLogs> {
        match self {
            ProviderApi::BosLife(api) => api.get_sub_logs().await,
        }
    }
}

use crate::config::client_config::ProxyClient;
use crate::config::provider_config::{Provider, ProviderConfig};
use crate::provider_api::provider_api_trait::ProviderApiTrait;
use headers::UserAgent;
use redis::aio::ConnectionManager;
use reqwest::Client as ReqwestClient;
use std::collections::HashMap;
use url::Url;

use crate::provider_api::boslife_api::BosLifeApi;
pub use api_response::*;
pub use boslife_api::{BosLifeLog, BosLifeLogs};

mod api_response;
mod boslife_api;
mod provider_api_trait;

type Result<T> = core::result::Result<T, crate::error::ProviderApiError>;

#[derive(Clone)]
pub enum ProviderApi {
    BosLife(BosLifeApi),
}

impl ProviderApi {
    pub fn create_api(
        providers: HashMap<Provider, ProviderConfig>,
        redis: ConnectionManager,
    ) -> HashMap<Provider, ProviderApi> {
        providers
            .into_iter()
            .map(|(provider, config)| match &provider {
                Provider::BosLife => (
                    provider,
                    ProviderApi::BosLife(BosLifeApi::new(config, Some(redis.clone()))),
                ),
            })
            .collect()
    }

    pub fn create_api_no_redis(providers: HashMap<Provider, ProviderConfig>) -> HashMap<Provider, ProviderApi> {
        providers
            .into_iter()
            .map(|(provider, config)| match &provider {
                Provider::BosLife => (provider, ProviderApi::BosLife(BosLifeApi::new(config, None))),
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

    pub async fn login(&self) -> Result<String> {
        match self {
            ProviderApi::BosLife(api) => api.login().await,
        }
    }

    // sub_url 是订阅地址并且应该指定客户端
    pub async fn get_raw_profile(&self, client: ProxyClient, user_agent: UserAgent) -> Result<String> {
        match self {
            ProviderApi::BosLife(api) => api.get_raw_profile(client, user_agent).await,
        }
    }

    pub async fn get_sub_url(&self) -> Result<Url> {
        match self {
            ProviderApi::BosLife(api) => api.get_sub_url().await,
        }
    }

    pub async fn reset_sub_url(&self) -> Result<Url> {
        match self {
            ProviderApi::BosLife(api) => api.reset_sub_url().await,
        }
    }

    pub async fn get_sub_logs(&self) -> Result<BosLifeLogs> {
        match self {
            ProviderApi::BosLife(api) => api.get_sub_logs().await,
        }
    }
}

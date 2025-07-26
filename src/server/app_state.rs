use crate::api::SubProviderWrapper;
use crate::common::config::ConvertorConfig;
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::sub_provider::SubProvider;
use crate::core::profile::policy::Policy;
use crate::server::clash_service::ClashService;
use crate::server::surge_service::SurgeService;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

pub struct AppState {
    pub config: Arc<ConvertorConfig>,
    pub api_map: HashMap<SubProvider, SubProviderWrapper>,
    pub surge_service: SurgeService,
    pub clash_service: ClashService,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProfileCacheKey {
    pub client: ProxyClient,
    pub provider: SubProvider,
    pub uni_sub_url: Url,
    pub interval: u64,
    pub server: Option<Url>,
    pub strict: Option<bool>,
    pub policy: Option<Policy>,
}

impl AppState {
    pub fn new(config: ConvertorConfig, api_map: HashMap<SubProvider, SubProviderWrapper>) -> Self {
        let config = Arc::new(config);
        let surge_service = SurgeService::new(config.clone());
        let clash_service = ClashService::new(config.clone());
        Self {
            config,
            api_map,
            surge_service,
            clash_service,
        }
    }
}

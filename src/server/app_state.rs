use crate::common::config::ConvertorConfig;
use crate::common::config::provider::SubProvider;
use crate::provider_api::ProviderApi;
use crate::server::service::{ClashService, SurgeService};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ConvertorConfig>,
    pub api_map: HashMap<SubProvider, ProviderApi>,
    pub surge_service: SurgeService,
    pub clash_service: ClashService,
}

impl AppState {
    pub fn new(config: ConvertorConfig, api_map: HashMap<SubProvider, ProviderApi>) -> Self {
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

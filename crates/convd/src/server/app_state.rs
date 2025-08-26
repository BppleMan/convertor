use crate::server::service::{ClashService, SurgeService};
use convertor::config::ConvertorConfig;
use convertor::config::provider_config::Provider;
use convertor::provider_api::ProviderApi;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ConvertorConfig>,
    pub api_map: HashMap<Provider, ProviderApi>,
    pub surge_service: SurgeService,
    pub clash_service: ClashService,
}

impl AppState {
    pub fn new(config: ConvertorConfig, api_map: HashMap<Provider, ProviderApi>) -> Self {
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

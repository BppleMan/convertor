use crate::server::service::{ClashService, SurgeService};
use convertor::config::ConvertorConfig;
use convertor::config::provider_config::Provider;
use convertor::provider_api::ProviderApi;
use redis::aio::ConnectionManager;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ConvertorConfig>,
    pub redis: Option<redis::Client>,
    pub redis_connection: Option<ConnectionManager>,
    pub api_map: HashMap<Provider, ProviderApi>,
    pub surge_service: SurgeService,
    pub clash_service: ClashService,
}

impl AppState {
    pub fn new(
        config: ConvertorConfig,
        redis: Option<redis::Client>,
        redis_connection: Option<ConnectionManager>,
    ) -> Self {
        let config = Arc::new(config);
        let surge_service = SurgeService::new(config.clone());
        let clash_service = ClashService::new(config.clone());
        let api_map = match redis_connection.clone() {
            None => ProviderApi::create_api_no_redis(config.providers.clone()),
            Some(redis_connection) => ProviderApi::create_api(config.providers.clone(), redis_connection),
        };
        Self {
            config,
            api_map,
            redis,
            redis_connection,
            surge_service,
            clash_service,
        }
    }
}

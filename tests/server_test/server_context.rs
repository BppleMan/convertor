use axum::Router;
use convertor::boslife::boslife_service::BosLifeService;

pub struct ServerContext {
    pub app: Router,
    pub service: BosLifeService,
}

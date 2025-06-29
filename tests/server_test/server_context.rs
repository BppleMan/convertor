use axum::Router;
use convertor::server::route::AppState;
use std::sync::Arc;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

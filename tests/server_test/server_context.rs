use axum::Router;
use convertor::server::router::AppState;
use std::sync::Arc;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

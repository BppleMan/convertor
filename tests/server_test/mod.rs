use axum::Router;
use convertor::server::router::AppState;
use httpmock::MockServer;
use std::path::PathBuf;
use std::sync::Arc;

pub mod surge_test;
pub mod clash_test;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
    pub mock_server: MockServer,
    pub base_dir: PathBuf,
}

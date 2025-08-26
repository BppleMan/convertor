use axum::Router;
use axum::routing::get;
use convd::server::app_state::AppState;
use convd::server::router::{profile, raw_profile, rule_provider};
use convertor::config::ConvertorConfig;
use convertor::testkit::start_mock_provider_server;
use std::sync::Arc;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

pub async fn start_server() -> color_eyre::Result<ServerContext> {
    let mut config = ConvertorConfig::template();
    start_mock_provider_server(&mut config).await?;

    let app_state = Arc::new(AppState::new(config, None, None));
    let app: Router = Router::new()
        .route("/raw-profile/{client}/{provider}", get(raw_profile))
        .route("/profile/{client}/{provider}", get(profile))
        .route("/rule-provider/{client}/{provider}", get(rule_provider))
        .with_state(app_state.clone());

    Ok(ServerContext { app, app_state })
}

use axum::Router;
use axum::routing::get;
use convd::server::app_state::AppState;
use convd::server::router::{api, profile};
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
        .route("/raw-profile/{client}/{provider}", get(profile::raw_profile))
        .route("/profile/{client}/{provider}", get(profile::profile))
        .route("/rule-provider/{client}/{provider}", get(profile::rule_provider))
        .route(
            "/api/subscription/{client}/{provider}",
            get(api::subscription::subscription),
        )
        .with_state(app_state.clone());

    Ok(ServerContext { app, app_state })
}

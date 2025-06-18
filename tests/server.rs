use crate::server_test::server_context::ServerContext;
use axum::routing::get;
use axum::Router;
use convertor::boslife::boslife_service::BosLifeService;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::server::route;
use convertor::server::route::AppState;
use std::sync::Arc;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;

pub mod server_test;

const TEST_CONFIG_STR: &str = include_str!("../test-assets/convertor.toml");

pub async fn start_server() -> color_eyre::Result<ServerContext> {
    color_eyre::install()?;

    let client = reqwest::Client::new();
    let config = ConvertorConfig::from_str(TEST_CONFIG_STR)?;
    let service = BosLifeService::new(client, config);

    let app_state = Arc::new(AppState {
        service: service.clone(),
    });
    let app: Router = Router::new()
        .route("/", get(route::root))
        .route("/clash", get(route::clash::profile))
        .route("/surge", get(route::surge::profile))
        .route("/surge/rule_set", get(route::surge::rule_set))
        .route(
            "/subscription_log",
            get(route::subscription::subscription_log),
        )
        .with_state(app_state.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    Ok(ServerContext { app, service })
}

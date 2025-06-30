use crate::server_test::server_context::ServerContext;
use axum::routing::get;
use axum::Router;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::server::router;
use convertor::server::router::AppState;
use convertor::subscription::subscription_api::boslife_api::BosLifeApi;
use convertor::subscription::subscription_config::ServiceConfig;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::warn;

pub mod server_test;

pub(crate) const TEST_CONFIG_STR: &str = include_str!("../test-assets/convertor.toml");
pub(crate) const CLASH_MOCK_STR: &str = include_str!("../test-assets/clash/mock.yaml");
pub(crate) const SURGE_MOCK_STR: &str = include_str!("../test-assets/surge/mock.conf");

pub async fn start_server_with_config(
    flag: impl AsRef<str>,
    config: Option<ConvertorConfig>,
) -> color_eyre::Result<ServerContext> {
    if let Err(e) = color_eyre::install() {
        warn!("Failed to install color_eyre: {}", e);
    }

    let client = reqwest::Client::new();
    let mut config = config
        .map(Ok)
        .unwrap_or_else(|| ConvertorConfig::from_str(TEST_CONFIG_STR))?;
    let mock_server = start_mock_service_server(flag, &config.service_config).await?;
    config.service_config.base_url = mock_server.base_url();

    let service = BosLifeApi::new(client, config.service_config.clone());

    let app_state = Arc::new(AppState {
        convertor_config: config,
        subscription_api: service,
    });
    let app: Router = Router::new()
        .route("/", get(router::root))
        .route("/surge", get(router::surge::profile))
        .route("/surge/rule-set", get(router::surge::rule_set))
        .route("/clash", get(router::clash::profile))
        .route("/clash/rule-set", get(router::clash::rule_set))
        .route("/subscription_log", get(router::subscription::subscription_logs))
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

    Ok(ServerContext { app, app_state })
}

pub async fn start_server(flag: impl AsRef<str>) -> color_eyre::Result<ServerContext> {
    start_server_with_config(flag, None).await
}

pub async fn start_mock_service_server(
    flag: impl AsRef<str>,
    config: &ServiceConfig,
) -> color_eyre::Result<MockServer> {
    if let Err(e) = color_eyre::install() {
        warn!("Failed to install color_eyre: {}", e);
    }

    let flag = flag.as_ref();

    let mock_server = MockServer::start_async().await;
    mock_server
        .mock_async(|when, then| {
            when.method(POST)
                .path(format!("{}{}", config.prefix_path, config.login_api.api_path));
            let body = serde_json::json!({
                "data": {
                    "auth_data": "mock_auth_token"
                }
            });
            then.status(200)
                .body(serde_json::to_string(&body).unwrap())
                .header("Content-Type", "application/json");
        })
        .await;

    let get_subscription_api_path = format!("{}{}", config.prefix_path, config.get_subscription_api.api_path);
    mock_server
        .mock_async(|when, then| {
            when.method(GET).path(get_subscription_api_path);
            let body = serde_json::json!({
                "data": {
                    "subscribe_url": mock_server.url("/subscription?token=bppleman"),
                }
            });
            then.status(200)
                .body(serde_json::to_string(&body).unwrap())
                .header("Content-Type", "application/json");
        })
        .await;
    mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/subscription")
                .query_param("flag", flag)
                .query_param("token", "bppleman");
            let body = if flag == "surge" {
                SURGE_MOCK_STR
            } else if flag == "clash" {
                CLASH_MOCK_STR
            } else {
                warn!("Unknown flag: {}", flag);
                return;
            };
            then.status(200)
                .body(body)
                .header("Content-Type", "text/plain; charset=utf-8");
        })
        .await;

    Ok(mock_server)
}

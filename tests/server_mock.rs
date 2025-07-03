use crate::server_test::ServerContext;
use axum::routing::get;
use axum::Router;
use convertor::client::Client;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::init_backtrace;
use convertor::server::router::{profile, root, rule_set, subscription, AppState};
use convertor::subscription::subscription_api::boslife_api::BosLifeApi;
use convertor::subscription::subscription_config::ServiceConfig;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use std::path::PathBuf;
use std::sync::{Arc, Once};

pub mod server_test;

pub(crate) const CLASH_MOCK_STR: &str = include_str!("../test-assets/clash/mock.yaml");
pub(crate) const SURGE_MOCK_STR: &str = include_str!("../test-assets/surge/mock.conf");

static INITIALIZED_TEST: Once = Once::new();

pub fn init_test_base_dir() -> PathBuf {
    std::env::current_dir().unwrap().join(".convertor.test")
}

pub fn init_test() -> PathBuf {
    let base_dir = init_test_base_dir();
    INITIALIZED_TEST.call_once(|| {
        init_backtrace();
    });
    base_dir
}

pub async fn start_server_with_config(
    client: Client,
    config: Option<ConvertorConfig>,
) -> color_eyre::Result<ServerContext> {
    let base_dir = init_test();

    let mut config = config
        .map(color_eyre::Result::Ok)
        .unwrap_or_else(|| ConvertorConfig::search(&base_dir, Option::<&str>::None))?;
    let mock_server = start_mock_service_server(client, &config.service_config).await?;
    config.service_config.base_url = mock_server.base_url();

    let api = BosLifeApi::new(&base_dir, reqwest::Client::new(), config.service_config.clone());
    let app_state = Arc::new(AppState { config, api });
    let app: Router = Router::new()
        .route("/", get(root))
        .route("/profile", get(profile))
        .route("/rule-set", get(rule_set))
        .route("/sub-log", get(subscription::subscription_logs))
        .with_state(app_state.clone());

    Ok(ServerContext {
        app,
        app_state,
        base_dir,
    })
}

pub async fn start_server(client: Client) -> color_eyre::Result<ServerContext> {
    start_server_with_config(client, None).await
}

pub async fn start_mock_service_server(client: Client, config: &ServiceConfig) -> color_eyre::Result<MockServer> {
    let _base_dir = init_test();

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
                .query_param("flag", client.as_str())
                .query_param("token", "bppleman");
            let body = match client {
                Client::Surge => SURGE_MOCK_STR,
                Client::Clash => CLASH_MOCK_STR,
            };
            then.status(200)
                .body(body)
                .header("Content-Type", "text/plain; charset=utf-8");
        })
        .await;

    Ok(mock_server)
}

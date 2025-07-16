use crate::server_test::ServerContext;
use axum::Router;
use axum::routing::get;
use convertor::client::Client;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::init_backtrace;
use convertor::profile::core::policy::Policy;
use convertor::profile::core::rule::{Rule, RuleType};
use convertor::profile::renderer::Renderer;
use convertor::profile::renderer::clash_renderer::ClashRenderer;
use convertor::profile::renderer::surge_renderer::SurgeRenderer;
use convertor::router::subscription_router::subscription_logs;
use convertor::router::{AppState, profile, rule_provider};
use convertor::subscription::subscription_api::boslife_api::BosLifeApi;
use convertor::subscription::subscription_config::ServiceConfig;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use include_dir::{Dir, include_dir};
use regex::Regex;
use reqwest::IntoUrl;
use std::path::PathBuf;
use std::sync::{Arc, Once};
use url::Url;

pub mod server_test;

// const CLASH_MOCK_STR: &str = include_str!("../.convertor.test/clash/mock.yaml");
// const SURGE_MOCK_STR: &str = include_str!("../.convertor.test/surge/mock.conf");
// const CLASH_EXPECT_STR: &str = include_str!("../.convertor.test/clash/expect.yaml");
// const SURGE_EXPECT_STR: &str = include_str!("../.convertor.test/surge/expect.conf");
const SURGE_MOCK_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/.convertor.test/surge");
const CLASH_MOCK_DIR: Dir = include_dir!(".convertor.test/clash");

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
        .map(Ok)
        .unwrap_or_else(|| ConvertorConfig::search(&base_dir, Option::<&str>::None))?;
    let mock_server = start_mock_service_server(client, &config.service_config).await?;
    config.service_config.base_url = Url::parse(&mock_server.base_url())?;

    let api = BosLifeApi::new(&base_dir, reqwest::Client::new(), config.service_config.clone());
    let app_state = Arc::new(AppState { config, api });
    let app: Router = Router::new()
        .route("/profile", get(profile))
        .route("/rule-provider", get(rule_provider))
        .route("/sub-logs", get(subscription_logs))
        .with_state(app_state.clone());

    Ok(ServerContext {
        app,
        app_state,
        mock_server,
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

    let get_subscription_api_path = format!("{}{}", config.prefix_path, config.get_sub_api.api_path);
    // 将订阅地址导航至 mock server 的 /subscription 路径
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
    // hook mock server 的 /subscription 路径，返回相应的 mock 数据
    mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/subscription")
                .query_param("flag", client.as_str())
                .query_param("token", "bppleman");
            let body = mock_profile(client, &mock_server).expect("无法生成 mock 配置文件");
            then.status(200)
                .body(body)
                .header("Content-Type", "text/plain; charset=utf-8");
        })
        .await;

    Ok(mock_server)
}

pub fn mock_profile(client: Client, mock_server: &MockServer) -> color_eyre::Result<String> {
    let rule = Rule {
        rule_type: RuleType::Domain,
        value: Some(mock_server.url("")),
        policy: Policy::subscription_policy(),
        comment: None,
    };
    let content = get_included_str(client, "mock");
    match client {
        Client::Surge => {
            let mut lines = content.lines().collect::<Vec<_>>();
            let rule_line = SurgeRenderer::render_rule(&rule)?;
            if let Some(i) = lines.iter().position(|l| l.starts_with("[Rule]")) {
                lines.insert(i + 1, &rule_line)
            }
            Ok(lines.join("\n"))
        }
        Client::Clash => {
            let mut lines = content.lines().collect::<Vec<_>>();
            let rule_line = format!("    - {}", ClashRenderer::render_rule(&rule)?);
            if let Some(i) = lines.iter().position(|l| l.starts_with("rules:")) {
                lines.insert(i + 1, &rule_line)
            }
            Ok(lines.join("\n"))
        }
    }
}

pub fn expect_profile(client: Client, encrypted_raw_sub_url: impl AsRef<str>) -> String {
    get_included_str(client, "profile").replace("{raw_sub_url}", encrypted_raw_sub_url.as_ref())
}

pub fn count_rule_lines(client: Client, policy: &Policy) -> usize {
    // match client {
    //     Client::Surge => {
    //         let expect_policy = SurgeRenderer::render_policy(policy).expect("无法渲染 Surge 策略");
    //         let lines = SURGE_MOCK_STR.lines().collect::<Vec<_>>();
    //         lines
    //             .iter()
    //             .filter(|line| {
    //                 !line.starts_with("//")
    //                     && !line.starts_with("#")
    //                     && !line.starts_with(";")
    //                     && line.ends_with(&expect_policy)
    //             })
    //             .count()
    //     }
    //     Client::Clash => {
    //         let expect_policy = ClashRenderer::render_policy(policy).expect("无法渲染 Clash 策略");
    //         let lines = CLASH_MOCK_STR.lines().collect::<Vec<_>>();
    //         lines
    //             .iter()
    //             .filter(|line| {
    //                 !line.starts_with("//")
    //                     && !line.starts_with("#")
    //                     && !line.starts_with(";")
    //                     && line.ends_with(&format!("{expect_policy}'"))
    //             })
    //             .count()
    //     }
    // }
    0
}

pub fn get_included_str(client: Client, file_name: impl AsRef<str>) -> String {
    let ext = match client {
        Client::Surge => "conf",
        Client::Clash => "yaml",
    };
    match client {
        Client::Surge => SURGE_MOCK_DIR,
        Client::Clash => CLASH_MOCK_DIR,
    }
    .get_file(format!("{}.{}", file_name.as_ref(), ext))
    .unwrap_or_else(|| panic!("无法找到文件: {}", file_name.as_ref()))
    .contents_utf8()
    .unwrap_or_else(|| panic!("无法解析 {} 文件内容", file_name.as_ref()))
    .to_string()
}

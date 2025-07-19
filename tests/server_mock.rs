mod server_test;

use axum::Router;
use axum::routing::get;
use color_eyre::eyre::eyre;
use convertor::api::ServiceApi;
use convertor::common::config::{ConvertorConfig, ServiceConfig};
use convertor::common::proxy_client::ProxyClient;
use convertor::common::url::Url;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::rule::{Rule, RuleType};
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use convertor::init_backtrace;
use convertor::server::surge_router::sub_logs;
use convertor::server::{AppState, profile, rule_provider};
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use include_dir::{Dir, include_dir};
use std::path::PathBuf;
use std::sync::{Arc, Once};

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
    pub mock_server: MockServer,
    pub base_dir: PathBuf,
}

const SURGE_MOCK_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/.convertor.test/surge");
const CLASH_MOCK_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/.convertor.test/clash");

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
    client: ProxyClient,
    config: Option<ConvertorConfig>,
) -> color_eyre::Result<ServerContext> {
    let base_dir = init_test();

    let mut config = config
        .map(Ok)
        .unwrap_or_else(|| ConvertorConfig::search(&base_dir, Option::<&str>::None))?;
    let mock_server = start_mock_service_server(client, &mut config.service_config).await?;
    config.service_config.api_host = Url::parse(&mock_server.base_url())?;

    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir);
    let app_state = Arc::new(AppState::new(config, api));
    let app: Router = Router::new()
        .route("/profile", get(profile))
        .route("/rule-provider", get(rule_provider))
        .route("/sub-logs", get(sub_logs))
        .with_state(app_state.clone());

    Ok(ServerContext {
        app,
        app_state,
        mock_server,
        base_dir,
    })
}

pub async fn start_server(client: ProxyClient) -> color_eyre::Result<ServerContext> {
    start_server_with_config(client, None).await
}

pub async fn start_mock_service_server(
    client: ProxyClient,
    config: &mut ServiceConfig,
) -> color_eyre::Result<MockServer> {
    let _base_dir = init_test();

    let mock_server = MockServer::start_async().await;
    config
        .raw_sub_url
        .set_port(Some(mock_server.port()))
        .map_err(|_| eyre!("无法设置 mock server 端口"))?;
    mock_server
        .mock_async(|when, then| {
            when.method(POST)
                .path(format!("{}{}", config.api_prefix, config.login_api.api_path));
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

    let get_subscription_api_path = format!("{}{}", config.api_prefix, config.get_sub_api.api_path);
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

pub fn mock_profile(client: ProxyClient, mock_server: &MockServer) -> color_eyre::Result<String> {
    let rule = Rule {
        rule_type: RuleType::Domain,
        value: Some(mock_server.url("")),
        policy: Policy::subscription_policy(),
        comment: None,
    };
    let content = get_included_str(client, "mock");
    match client {
        ProxyClient::Surge => {
            let mut lines = content.lines().collect::<Vec<_>>();
            let rule_line = SurgeRenderer::render_rule(&rule)?;
            if let Some(i) = lines.iter().position(|l| l.starts_with("[Rule]")) {
                lines.insert(i + 1, &rule_line)
            }
            Ok(lines.join("\n"))
        }
        ProxyClient::Clash => {
            let mut lines = content.lines().collect::<Vec<_>>();
            let rule_line = format!("    - {}", ClashRenderer::render_rule(&rule)?);
            if let Some(i) = lines.iter().position(|l| l.starts_with("rules:")) {
                lines.insert(i + 1, &rule_line)
            }
            Ok(lines.join("\n"))
        }
    }
}

pub fn expect_profile(client: ProxyClient, encrypted_raw_sub_url: impl AsRef<str>) -> String {
    get_included_str(client, "profile").replace("{raw_sub_url}", encrypted_raw_sub_url.as_ref())
}

pub fn expect_rule_provider(client: ProxyClient, policy: &Policy) -> String {
    match client {
        // 统一用 ClashRenderer 渲染策略名称, 作为文件名更方便
        ProxyClient::Surge => get_included_str(client, ClashRenderer::render_provider_name_for_policy(policy).unwrap()),
        ProxyClient::Clash => get_included_str(client, ClashRenderer::render_provider_name_for_policy(policy).unwrap()),
    }
}

pub fn get_included_str(client: ProxyClient, file_name: impl AsRef<str>) -> String {
    let ext = match client {
        ProxyClient::Surge => "conf",
        ProxyClient::Clash => "yaml",
    };
    match client {
        ProxyClient::Surge => SURGE_MOCK_DIR,
        ProxyClient::Clash => CLASH_MOCK_DIR,
    }
    .get_file(format!("{}.{}", file_name.as_ref(), ext))
    .unwrap_or_else(|| panic!("无法找到文件: {}", file_name.as_ref()))
    .contents_utf8()
    .unwrap_or_else(|| panic!("无法解析 {} 文件内容", file_name.as_ref()))
    .to_string()
}

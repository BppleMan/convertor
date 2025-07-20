use crate::{CLASH_MOCK_DIR, SURGE_MOCK_DIR, init_test};
use axum::Router;
use axum::routing::get;
use color_eyre::Report;
use color_eyre::eyre::eyre;
use convertor::api::SubProviderApi;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::config::sub_provider::SubProviderConfig;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::rule::{Rule, RuleType};
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use convertor::server::surge_router::sub_logs;
use convertor::server::{AppState, profile, rule_provider};
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use moka::future::Cache;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, LazyLock};
use url::Url;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

pub async fn start_server_with_config() -> color_eyre::Result<ServerContext> {
    let base_dir = init_test();
    let mut config = ConvertorConfig::template()?;
    let mock_server = start_mock_provider_server(&mut config.provider).await?;
    config
        .provider
        .uni_sub_url
        .set_port(Some(mock_server.port()))
        .map_err(|_| eyre!("can't set mock server port"))?;
    config.provider.api_host = Url::parse(&mock_server.base_url())?;

    let api = SubProviderApi::get_service_provider_api(config.provider.clone(), &base_dir);
    let app_state = Arc::new(AppState::new(config, api));
    let app: Router = Router::new()
        .route("/profile", get(profile))
        .route("/rule-provider", get(rule_provider))
        .route("/sub-logs", get(sub_logs))
        .with_state(app_state.clone());

    Ok(ServerContext { app, app_state })
}

pub async fn start_server() -> color_eyre::Result<ServerContext> {
    start_server_with_config().await
}

static CACHED_MOCK_SERVER: LazyLock<Cache<u64, Arc<MockServer>>> =
    LazyLock::new(|| Cache::builder().max_capacity(100).build());

pub async fn start_mock_provider_server(config: &SubProviderConfig) -> Result<Arc<MockServer>, Report> {
    let mut hasher = DefaultHasher::default();
    config.hash(&mut hasher);
    let hash = hasher.finish();
    let mock_server = CACHED_MOCK_SERVER
        .try_get_with(hash, async {
            let _base_dir = init_test();

            let mock_server = MockServer::start_async().await;
            println!("Mock server started at: {}", mock_server.base_url());

            let login_api_path = format!("{}{}", config.api_prefix, config.login_api.path);
            mock_server
                .mock_async(|when, then| {
                    when.method(POST).path(login_api_path);
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

            let get_sub_url_api_path = format!("{}{}", config.api_prefix, config.get_sub_url_api.path);
            // 将订阅地址导航至 mock server 的 /subscription 路径
            mock_server
                .mock_async(|when, then| {
                    when.method(GET).path(get_sub_url_api_path);
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
                        .query_param("flag", ProxyClient::Surge.as_str())
                        .query_param("token", "bppleman");
                    let body = mock_profile(ProxyClient::Surge, &mock_server).expect("无法生成 mock 配置文件");
                    then.status(200)
                        .body(body)
                        .header("Content-Type", "text/plain; charset=utf-8");
                })
                .await;

            mock_server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path("/subscription")
                        .query_param("flag", ProxyClient::Clash.as_str())
                        .query_param("token", "bppleman");
                    let body = mock_profile(ProxyClient::Clash, &mock_server).expect("无法生成 mock 配置文件");
                    then.status(200)
                        .body(body)
                        .header("Content-Type", "text/plain; charset=utf-8");
                })
                .await;

            Ok::<_, Report>(Arc::new(mock_server))
        })
        .await
        .map_err(|e| eyre!(e))?;
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
    get_included_str(client, "expect")
        .replace("{uni_sub_url}", encrypted_raw_sub_url.as_ref())
        .replace("{CARGO_PKG_VERSION}", env!("CARGO_PKG_VERSION"))
}

pub fn expect_rule_provider(client: ProxyClient, policy: &Policy) -> String {
    match client {
        // 统一用 ClashRenderer 渲染策略名称, 作为文件名更方便
        ProxyClient::Surge => {
            // !!! 不要修改这里的 ClashRenderer
            get_included_str(client, ClashRenderer::render_provider_name_for_policy(policy).unwrap())
        }
        ProxyClient::Clash => get_included_str(client, ClashRenderer::render_provider_name_for_policy(policy).unwrap()),
    }
}

pub fn get_included_str(client: ProxyClient, file_name: impl AsRef<str>) -> String {
    let ext = match client {
        ProxyClient::Surge => "conf",
        ProxyClient::Clash => "yaml",
    };
    match client {
        ProxyClient::Surge => &SURGE_MOCK_DIR,
        ProxyClient::Clash => &CLASH_MOCK_DIR,
    }
    .get_file(format!("{}.{}", file_name.as_ref(), ext))
    .unwrap_or_else(|| panic!("无法找到文件: {}", file_name.as_ref()))
    .contents_utf8()
    .unwrap_or_else(|| panic!("无法解析 {} 文件内容", file_name.as_ref()))
    .to_string()
}

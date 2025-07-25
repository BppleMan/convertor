use crate::{CLASH_MOCK_DIR, SURGE_MOCK_DIR, init_test};
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::routing::get;
use color_eyre::Report;
use color_eyre::eyre::eyre;
use convertor::api::SubProviderWrapper;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::config::sub_provider::{BosLifeConfig, SubProvider, SubProviderConfig};
use convertor::core::profile::policy::Policy;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::url_builder::HostPort;
use convertor::server::surge_router::sub_logs;
use convertor::server::{AppState, profile, rule_provider};
use dispatch_map::DispatchMap;
use http_body_util::BodyExt;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use moka::future::Cache;
use rstest::{fixture, rstest};
use std::sync::{Arc, LazyLock};
use std::thread;
use tower::ServiceExt;
use url::Url;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

#[fixture]
#[once]
pub fn server_context() -> ServerContext {
    thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async { start_server().await.unwrap() })
    })
    .join()
    .unwrap()
}

#[rstest]
#[tokio::test]
pub async fn test_profile(
    server_context: &ServerContext,
    #[values(ProxyClient::Surge, ProxyClient::Clash)] client: ProxyClient,
    #[values(SubProvider::BosLife)] provider: SubProvider,
) -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client, provider)?;

    let convertor_url = url_builder.build_sub_url()?;
    let expect_placeholder = ExpectPlaceholder {
        server: convertor_url.server.to_string(),
        uni_sub_host: convertor_url.query.uni_sub_url.host_port()?,
        enc_uni_sub_url: convertor_url.query.encoded_uni_sub_url(),
    };
    let uri = format!(
        "{}?{}",
        convertor_url.path,
        convertor_url.query.encode_to_query_string()
    );

    let request = Request::builder().uri(uri).method("GET").body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_profile(client, &expect_placeholder);

    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[rstest]
#[tokio::test]
pub async fn test_rule_provider(
    server_context: &ServerContext,
    #[values(ProxyClient::Surge, ProxyClient::Clash)] client: ProxyClient,
    #[values(SubProvider::BosLife)] provider: SubProvider,
    #[values(
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::direct_policy(None),
        Policy::direct_policy(Some("no-resolve")),
        Policy::direct_policy(Some("force-remote-dns"))
    )]
    policy: Policy,
) -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client, provider)?;

    let convertor_url = url_builder.build_rule_provider_url(&policy)?;
    let expect_placeholder = ExpectPlaceholder {
        server: convertor_url.server.to_string(),
        uni_sub_host: convertor_url.query.uni_sub_url.host_port()?,
        enc_uni_sub_url: convertor_url.query.encoded_uni_sub_url(),
    };
    let uri = format!(
        "{}?{}",
        convertor_url.path,
        convertor_url.query.encode_to_query_string()
    );

    let request = Request::builder().uri(uri).method("GET").body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_rule_provider(client, &policy, &expect_placeholder);

    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

pub async fn start_server() -> color_eyre::Result<ServerContext> {
    let base_dir = init_test();
    let mut config = ConvertorConfig::template();
    start_mock_provider_server(&mut config.providers).await?;

    let api = SubProviderWrapper::create_api(config.providers.clone(), &base_dir);
    let app_state = Arc::new(AppState::new(config, api));
    let app: Router = Router::new()
        .route("/profile", get(profile))
        .route("/rule-provider", get(rule_provider))
        .route("/sub-logs", get(sub_logs))
        .with_state(app_state.clone());

    Ok(ServerContext { app, app_state })
}

static CACHED_MOCK_SERVER: LazyLock<Cache<SubProviderConfig, Arc<MockServer>>> =
    LazyLock::new(|| Cache::builder().max_capacity(100).build());

pub async fn start_mock_provider_server(
    providers: &mut DispatchMap<SubProvider, SubProviderConfig>,
) -> Result<(), Report> {
    init_test();
    for (_, config) in providers.iter_mut() {
        CACHED_MOCK_SERVER
            .try_get_with(config.clone(), async {
                let mock_server = match config {
                    SubProviderConfig::BosLife(config) => config.start_mock_provider_server().await?,
                };
                Ok::<_, Report>(Arc::new(mock_server))
            })
            .await
            .map_err(|e| eyre!(e))?;
    }
    Ok(())
}

pub(crate) trait MockServerExt {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report>;
}

impl MockServerExt for BosLifeConfig {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report> {
        let mock_server = MockServer::start_async().await;
        println!("Mock 服务器启动: {}", mock_server.base_url());

        // 将订阅地址导航至 mock server 的 /subscription 路径
        let subscribe_url_path = "/subscription";
        let token = "bppleman";

        self.uni_sub_url =
            Url::parse(&mock_server.url(format!("{subscribe_url_path}?token={token}"))).expect("不合法的订阅地址");
        self.api_host = Url::parse(&mock_server.base_url())?;

        let mock_placeholder = MockPlaceholder {
            uni_sub_host: self.uni_sub_url.host_port()?,
        };

        mock_server
            .mock_async(|when, then| {
                when.method(POST).path(self.login_path());
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

        mock_server
            .mock_async(|when, then| {
                when.method(GET).path(self.get_sub_url_path());
                let body = serde_json::json!({
                    "data": {
                        "subscribe_url": mock_server.url(format!("{subscribe_url_path}?token={token}")),
                    }
                });
                then.status(200)
                    .body(serde_json::to_string(&body).unwrap())
                    .header("Content-Type", "application/json");
            })
            .await;

        // hook mock server 的 /subscription 路径，返回相应的 mock 数据
        for client in ProxyClient::clients() {
            mock_server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path(subscribe_url_path)
                        .query_param("flag", client.as_str())
                        .query_param("token", token);
                    let body = mock_profile(client, &mock_placeholder);
                    then.status(200)
                        .body(body)
                        .header("Content-Type", "text/plain; charset=utf-8");
                })
                .await;
        }

        Ok(mock_server)
    }
}

pub struct MockPlaceholder {
    pub uni_sub_host: String,
}

pub struct ExpectPlaceholder {
    pub server: String,
    pub uni_sub_host: String,
    pub enc_uni_sub_url: String,
}

pub fn mock_profile(client: ProxyClient, placeholder: &MockPlaceholder) -> String {
    get_included_str(client, "mock").replace("{uni_sub_host}", &placeholder.uni_sub_host)
}

pub fn expect_profile(client: ProxyClient, expect_placeholder: &ExpectPlaceholder) -> String {
    get_included_str(client, "expect")
        .replace("{server}", &expect_placeholder.server)
        .replace("{uni_sub_url}", &expect_placeholder.enc_uni_sub_url)
        .replace("{CARGO_PKG_VERSION}", env!("CARGO_PKG_VERSION"))
}

pub fn expect_rule_provider(client: ProxyClient, policy: &Policy, expect_placeholder: &ExpectPlaceholder) -> String {
    // 统一用 ClashRenderer 渲染策略名称, 作为文件名更方便
    let name = ClashRenderer::render_provider_name_for_policy(policy).unwrap();
    get_included_str(client, format!("rule_providers/{name}"))
        .replace("{uni_sub_host}", &expect_placeholder.uni_sub_host)
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

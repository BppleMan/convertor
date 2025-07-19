use crate::api::ServiceApi;
use crate::common::config::ConvertorConfig;
use crate::common::proxy_client::ProxyClient;
use crate::common::url::{ConvertorUrl, ConvertorUrlError};
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::surge_profile::SurgeProfile;
use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::StatusCode;
use axum::http::header::ToStrError;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use moka::future::Cache;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use surge_router::sub_logs;
use thiserror::Error;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::instrument;
use tracing::{info, warn};

pub mod clash_router;
pub mod surge_router;
pub mod query;

#[derive(Debug, Parser)]
#[clap(version, author)]
pub struct ConvertorServer {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8001")]
    pub listen: SocketAddrV4,

    #[arg(short)]
    pub config: Option<PathBuf>,
}

pub struct AppState {
    pub config: ConvertorConfig,
    pub api: ServiceApi,
    pub profile_cache: Cache<ConvertorUrl, String>,
    pub surge_cache: Cache<ConvertorUrl, SurgeProfile>,
    pub clash_cache: Cache<ConvertorUrl, ClashProfile>,
}

impl AppState {
    pub fn new(config: ConvertorConfig, api: ServiceApi) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60); // 1 hour
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        let surge_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        let clash_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self {
            config,
            api,
            profile_cache,
            surge_cache,
            clash_cache,
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    ConvertorUrl(#[from] ConvertorUrlError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    Eyre(#[from] color_eyre::eyre::Error),

    #[error(transparent)]
    ToStr(#[from] ToStrError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    CacheError(#[from] Arc<AppError>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = format!("{self:?}");
        let message = console::strip_ansi_codes(&message).to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}

pub async fn start_server(
    listen_addr: SocketAddrV4,
    config: ConvertorConfig,
    api: ServiceApi,
    base_dir: impl AsRef<Path>,
) -> Result<()> {
    info!("base_dir: {}", base_dir.as_ref().display());
    info!("监听中: {}", &listen_addr);
    warn!("建议使用 nginx 等网关进行反向代理，以开启 HTTPS 支持");
    info!("服务启动，使用 Ctrl+C 或 SIGTERM 关闭服务");
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    let app_state = AppState::new(config, api);
    let app = router(app_state);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("服务关闭");
    Ok(())
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/profile", get(profile))
        .route("/rule-provider", get(rule_provider))
        .route("/sub-logs", get(sub_logs))
        .with_state(Arc::new(app_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
}

#[instrument(skip_all)]
pub async fn profile(State(state): State<Arc<AppState>>, RawQuery(query): RawQuery) -> Result<String, AppError> {
    let convertor_url = query
        .map(|query| ConvertorUrl::parse_from_query_string(query, &state.config.secret))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let client = convertor_url.client;
    let profile = state
        .clone()
        .profile_cache
        .try_get_with(convertor_url.clone(), async {
            let raw_profile = state.api.get_raw_profile(client).await?;
            let profile = match client {
                ProxyClient::Surge => surge_router::profile_impl(state, convertor_url, raw_profile).await,
                ProxyClient::Clash => clash_router::profile_impl(state, convertor_url, raw_profile).await,
            }?;
            Ok::<_, AppError>(profile)
        })
        .await?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(State(state): State<Arc<AppState>>, RawQuery(query): RawQuery) -> Result<String, AppError> {
    let convertor_url = query
        .map(|query| ConvertorUrl::parse_from_query_string(query, &state.config.secret))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let client = &convertor_url.client;
    let policy = convertor_url.policy.clone();
    let raw_profile = state.api.get_raw_profile(*client).await?;
    match (client, policy) {
        (ProxyClient::Surge, Some(policy)) => {
            surge_router::rule_provider_impl(state, convertor_url, raw_profile, policy.into()).await
        }
        (ProxyClient::Clash, Some(policy)) => {
            clash_router::rule_provider_impl(state, convertor_url, raw_profile, policy.into()).await
        }
        _ => Err(eyre!("错误的 client 或 policy 参数")),
    }
    .map_err(Into::into)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
pub(crate) mod server_mock {
    use crate::api::ServiceApi;
    use crate::common::config::{ConvertorConfig, ServiceConfig};
    use crate::common::once::init_backtrace;
    use crate::common::proxy_client::ProxyClient;
    use crate::core::profile::policy::Policy;
    use crate::core::profile::rule::{Rule, RuleType};
    use crate::core::renderer::Renderer;
    use crate::core::renderer::clash_renderer::ClashRenderer;
    use crate::core::renderer::surge_renderer::SurgeRenderer;
    use crate::server::surge_router::sub_logs;
    use crate::server::{AppState, profile, rule_provider};
    use axum::Router;
    use axum::routing::get;
    use color_eyre::eyre::eyre;
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use include_dir::{Dir, include_dir};
    use std::path::PathBuf;
    use std::sync::{Arc, Once};
    use url::Url;

    pub struct ServerContext {
        pub app: Router,
        pub app_state: Arc<AppState>,
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

        Ok(ServerContext { app, app_state })
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
            ProxyClient::Surge => {
                get_included_str(client, ClashRenderer::render_provider_name_for_policy(policy).unwrap())
            }
            ProxyClient::Clash => {
                get_included_str(client, ClashRenderer::render_provider_name_for_policy(policy).unwrap())
            }
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
}

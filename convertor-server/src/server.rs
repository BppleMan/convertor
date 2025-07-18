use axum::Router;
use axum::extract::{RawQuery, State};
use axum::http::StatusCode;
use axum::http::header::ToStrError;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use convertor_core::api::ServiceApi;
use convertor_core::client::Client;
use convertor_core::config::ConvertorConfig;
use convertor_core::core::profile::clash_profile::ClashProfile;
use convertor_core::core::profile::surge_profile::SurgeProfile;
use convertor_core::url::{ConvertorUrl, ConvertorUrlError};
use moka::future::Cache;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::Path;
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
                Client::Surge => surge_router::profile_impl(state, convertor_url, raw_profile).await,
                Client::Clash => clash_router::profile_impl(state, convertor_url, raw_profile).await,
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
        (Client::Surge, Some(policy)) => {
            surge_router::rule_provider_impl(state, convertor_url, raw_profile, policy.into()).await
        }
        (Client::Clash, Some(policy)) => {
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

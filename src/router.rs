use crate::client::Client;
use crate::convertor_config::ConvertorConfig;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::error::AppError;
use crate::router::query::ProfileQuery;
use crate::router::subscription_router::subscription_logs;
use crate::service_provider::api::ServiceApi;
use crate::shutdown_signal;
use crate::url_builder::UrlBuilder;
use axum::Router;
use axum::extract::{RawQuery, State};
use axum::routing::get;
use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use moka::future::Cache;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::Path;
use std::sync::Arc;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::instrument;
use tracing::{info, warn};

pub mod query;
pub mod clash_router;
pub mod surge_router;
pub mod subscription_router;

pub struct AppState {
    pub config: ConvertorConfig,
    pub api: ServiceApi,
    pub surge_cache: Cache<String, SurgeProfile>,
    pub clash_cache: Cache<String, ClashProfile>,
}

impl AppState {
    pub fn new(config: ConvertorConfig, api: ServiceApi) -> Self {
        let surge_cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(60 * 60))
            .build();
        let clash_cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(60 * 60))
            .build();
        Self {
            config,
            api,
            surge_cache,
            clash_cache,
        }
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
        .route("/sub-logs", get(subscription_logs))
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
    let profile_query = query
        .map(ProfileQuery::decode_from_query_string)
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let url_builder = UrlBuilder::decode_from_query(&profile_query, &state.config.secret)?;
    let raw_profile = state.api.get_raw_profile(profile_query.client).await?;
    let profile = match profile_query.client {
        Client::Surge => surge_router::profile_impl(state, url_builder, raw_profile).await,
        Client::Clash => clash_router::profile_impl(state, url_builder, raw_profile).await,
    }?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(State(state): State<Arc<AppState>>, RawQuery(query): RawQuery) -> Result<String, AppError> {
    let profile_query = query
        .map(ProfileQuery::decode_from_query_string)
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let url_builder = UrlBuilder::decode_from_query(&profile_query, &state.config.secret)?;
    let raw_profile = state.api.get_raw_profile(profile_query.client).await?;
    match (profile_query.client, profile_query.policy) {
        (Client::Surge, Some(policy)) => {
            surge_router::rule_provider_impl(state, url_builder, raw_profile, policy.into()).await
        }
        (Client::Clash, Some(policy)) => {
            clash_router::rule_provider_impl(state, url_builder, raw_profile, policy.into()).await
        }
        _ => Err(eyre!("错误的 client 或 policy 参数")),
    }
    .map_err(Into::into)
}

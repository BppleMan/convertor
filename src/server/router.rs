use crate::client::Client;
use crate::config::convertor_config::ConvertorConfig;
use crate::error::AppError;
use crate::profile::core::policy::QueryPolicy;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::url_builder::UrlBuilder;
use axum::body::Body;
use axum::extract::{Query, RawQuery, State};
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::instrument;

pub mod clash;
pub mod surge;
pub mod subscription;

pub struct AppState {
    pub config: ConvertorConfig,
    pub api: BosLifeApi,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileQuery {
    pub client: Client,
    pub raw_url: String,
    #[serde(default)]
    pub policy: Option<QueryPolicy>,
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/profile", get(profile))
        .route("/rule-set", get(rule_set))
        .route("/sub-log", get(subscription::subscription_logs))
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

#[instrument]
pub async fn root() -> Result<(), AppError> {
    Err(eyre!("Hello, World!"))?;
    Ok(())
}

#[instrument(skip(state))]
pub async fn profile(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    // info!("请求配置文件");
    // info!("从 request 中解码 url_builder");
    let url_builder = UrlBuilder::decode_from_request(&request, &state.config.secret)?;
    // info!("构建订阅 url");
    let raw_subscription_url = url_builder.build_subscription_url(query.client)?;
    // info!("获取原始订阅内容");
    let raw_profile = state.api.get_raw_profile(raw_subscription_url, query.client).await?;
    // info!("整理订阅内容");
    let profile = match query.client {
        Client::Surge => surge::profile_impl(state, url_builder, raw_profile).await,
        Client::Clash => clash::profile_impl(state, url_builder, raw_profile).await,
    }?;
    Ok(profile)
}

#[instrument(skip(state))]
pub async fn rule_set(
    State(state): State<Arc<AppState>>,
    // Query(query): Query<ProfileQuery>,
    RawQuery(query): RawQuery,
    request: Request<Body>,
) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(&request, &state.config.secret)?;
    let query = query
        .as_ref()
        .map(|q| percent_decode_str(&q))
        .ok_or_else(|| eyre!("缺少查询参数"))?
        .decode_utf8()
        .map_err(|e| eyre!(e))?;
    let query = serde_qs::from_str::<ProfileQuery>(&query).map_err(|e| eyre!(e))?;
    let raw_subscription_url = url_builder.build_subscription_url(query.client)?;
    let raw_profile = state.api.get_raw_profile(raw_subscription_url, query.client).await?;
    match (query.client, query.policy) {
        (Client::Surge, Some(policy)) => surge::rule_set_impl(state, url_builder, raw_profile, policy.into()).await,
        (Client::Clash, Some(policy)) => clash::rule_set_impl(state, url_builder, raw_profile, policy.into()).await,
        _ => Err(eyre!("错误的 client 或 policy 参数")),
    }
    .map_err(Into::into)
}

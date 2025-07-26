use crate::common::config::proxy_client::ProxyClient;
use crate::core::query::convertor_query::ConvertorQuery;
use crate::core::query::rule_provider_query::RuleProviderQuery;
use crate::server::error::AppError;
use crate::server::surge_router::sub_logs;
use crate::server::{AppState, clash_router, surge_router};
use axum::Router;
use axum::extract::{RawQuery, State};
use axum::response::Html;
use axum::routing::get;
use color_eyre::eyre::{WrapErr, eyre};
use std::sync::Arc;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::instrument;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Html(include_str!("../../assets/index.html")) }))
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

// pub fn root() -> String {}

#[instrument(skip_all)]
pub async fn profile(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> color_eyre::Result<String, AppError> {
    let query = query
        .map(|query| ConvertorQuery::parse_from_query_string(query, &state.config.secret))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let client = query.client;
    let Some(api) = state.api_map.get(&query.provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(client).await?;
    let profile = match client {
        ProxyClient::Surge => surge_router::profile_impl(state, query, raw_profile).await,
        ProxyClient::Clash => clash_router::profile_impl(state, query, raw_profile).await,
    }?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> color_eyre::Result<String, AppError> {
    let query = query
        .map(|query| RuleProviderQuery::parse_from_query_string(query, &state.config.secret))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let client = &query.client;
    let policy = query.policy.clone();
    let Some(api) = state.api_map.get(&query.provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(*client).await?;
    let rules = match client {
        ProxyClient::Surge => surge_router::rule_provider_impl(state, query, raw_profile, policy.into()).await,
        ProxyClient::Clash => clash_router::rule_provider_impl(state, query, raw_profile, policy.into()).await,
    }?;
    Ok(rules)
}

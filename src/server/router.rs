use crate::api::boslife_sub_log::BosLifeSubLog;
use crate::common::config::proxy_client::ProxyClient;
use crate::server::AppState;
use crate::server::error::AppError;
use crate::server::query::profile_query::ProfileQuery;
use crate::server::query::rule_provider_query::RuleProviderQuery;
use crate::server::query::sub_logs_query::SubLogsQuery;
use axum::extract::{RawQuery, State};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use color_eyre::eyre::{OptionExt, WrapErr, eyre};
use std::sync::Arc;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::instrument;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Html(include_str!("../../assets/index.html")) }))
        .route("/ready", get(|| async { "ok" }))
        .route("/healthy", get(|| async { "ok" }))
        .route("/raw-profile", get(raw_profile))
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

pub async fn raw_profile(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> color_eyre::Result<String, AppError> {
    let query = query
        .map(|query| ProfileQuery::parse_from_query_string(query, &state.config.secret))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let client = query.client;
    let Some(api) = state.api_map.get(&query.provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(client).await?;
    Ok(raw_profile)
}

#[instrument(skip_all)]
pub async fn profile(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> color_eyre::Result<String, AppError> {
    let query = query
        .map(|query| ProfileQuery::parse_from_query_string(query, &state.config.secret))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let client = query.client;
    let Some(api) = state.api_map.get(&query.provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(client).await?;
    let profile = match client {
        ProxyClient::Surge => state.surge_service.profile(query, raw_profile).await,
        ProxyClient::Clash => state.clash_service.profile(query, raw_profile).await,
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
    let Some(api) = state.api_map.get(&query.provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(*client).await?;
    let rules = match client {
        ProxyClient::Surge => state.surge_service.rule_provider(query, raw_profile).await,
        ProxyClient::Clash => state.clash_service.rule_provider(query, raw_profile).await,
    }?;
    Ok(rules)
}

pub async fn sub_logs(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> color_eyre::Result<Json<Vec<BosLifeSubLog>>, AppError> {
    let query = query.as_ref().ok_or_eyre(eyre!("订阅记录必须传递参数"))?;
    let sub_log_query = SubLogsQuery::decode_from_query_string(query, &state.config.secret)?;
    let provider = sub_log_query.provider;
    let Some(api) = state.api_map.get(&provider) else {
        return Err(AppError::NoSubProvider);
    };
    let logs = state.surge_service.sub_logs(sub_log_query, api).await?;
    Ok(Json(logs))
}

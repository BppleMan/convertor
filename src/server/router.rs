use crate::client::Client;
use crate::config::convertor_config::ConvertorConfig;
use crate::error::AppError;
use crate::server::query::ProfileQuery;
use crate::server::router::subscription_router::subscription_logs;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::url_builder::UrlBuilder;
use axum::extract::{RawQuery, State};
use axum::routing::get;
use axum::Router;
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Result;
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::instrument;

pub mod clash_router;
pub mod surge_router;
pub mod subscription_router;

pub struct AppState {
    pub config: ConvertorConfig,
    pub api: BosLifeApi,
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
    println!("raw query: {:?}", query);
    let profile_query = query
        .map(ProfileQuery::decode_from_query_string)
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let url_builder = UrlBuilder::decode_from_query(&profile_query, &state.config.secret)?;
    let raw_subscription_url = url_builder.build_subscription_url(profile_query.client)?;
    let raw_profile = state
        .api
        .get_raw_profile(raw_subscription_url, profile_query.client)
        .await?;
    let profile = match profile_query.client {
        Client::Surge => surge_router::profile_impl(state, url_builder, raw_profile).await,
        Client::Clash => clash_router::profile_impl(state, url_builder, raw_profile).await,
    }?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(State(state): State<Arc<AppState>>, RawQuery(query): RawQuery) -> Result<String, AppError> {
    println!("raw query: {:?}", query);
    let profile_query = query
        .map(ProfileQuery::decode_from_query_string)
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    let url_builder = UrlBuilder::decode_from_query(&profile_query, &state.config.secret)?;
    let raw_subscription_url = url_builder.build_subscription_url(profile_query.client)?;
    let raw_profile = state
        .api
        .get_raw_profile(raw_subscription_url, profile_query.client)
        .await?;
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

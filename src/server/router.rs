use crate::config::convertor_config::ConvertorConfig;
use crate::error::AppError;
use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use crate::subscription::url_builder::UrlBuilder;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::instrument;

pub mod clash;
pub mod surge;
pub mod subscription;

pub struct AppState {
    pub convertor_config: ConvertorConfig,
    pub subscription_api: BosLifeApi,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileQuery {
    pub flag: String,
    pub raw_url: String,
    #[serde(default)]
    pub policy: Option<RuleSetPolicy>,
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
    let url_builder = UrlBuilder::decode_from_request(&request, &state.convertor_config.secret)?;
    let raw_subscription_url = url_builder.build_subscription_url(&query.flag)?;
    let raw_profile = state.subscription_api.get_raw_profile(raw_subscription_url).await?;
    match query.flag.as_str() {
        "surge" => surge::profile_impl(state, url_builder, raw_profile).await,
        "clash" => clash::profile_impl(state, url_builder, raw_profile).await,
        _ => Err(eyre!("未知的 flag: {}", query.flag).into()),
    }
    .map_err(Into::into)
}

#[instrument(skip(state))]
pub async fn rule_set(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(&request, &state.convertor_config.secret)?;
    let raw_subscription_url = url_builder.build_subscription_url(&query.flag)?;
    let raw_profile = state.subscription_api.get_raw_profile(raw_subscription_url).await?;
    match query.flag.as_str() {
        "surge" => surge::rule_set_impl(state, url_builder, raw_profile, query.policy.clone()).await,
        "clash" => clash::rule_set_impl(state, url_builder, raw_profile, query.policy.clone()).await,
        _ => Err(eyre!("未知的 flag: {}", query.flag).into()),
    }
    .map_err(Into::into)
}

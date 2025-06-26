use crate::config::url_builder::UrlBuilder;
use crate::error::AppError;
use crate::profile::surge_profile::SurgeProfile;
use crate::server::route::AppState;
use crate::service::service_api::ServiceApi;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::Request;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SurgeQuery {
    pub raw_url: String,
    #[serde(default)]
    pub policies: Option<String>,
    #[serde(default)]
    pub boslife: Option<bool>,
}

pub async fn profile(
    State(state): State<Arc<AppState>>,
    query: Query<SurgeQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let convertor_url = UrlBuilder::decode_from_request(
        &request,
        &state.service.config.secret,
    )?;
    profile_impl(state, convertor_url).await.map_err(Into::into)
}

/// policy:
/// - Direct
/// - BosLife
/// - no-resolve
/// - force-remote-dns
pub async fn rule_set(
    State(state): State<Arc<AppState>>,
    query: Query<SurgeQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let convertor_url = UrlBuilder::decode_from_request(
        &request,
        &state.service.config.secret,
    )?;
    rule_set_impl(state, convertor_url, query.policies.clone(), query.boslife)
        .await
        .map_err(Into::into)
}

async fn profile_impl(
    state: Arc<AppState>,
    convertor_url: UrlBuilder,
) -> Result<String> {
    let raw_profile = state
        .service
        .get_raw_profile(convertor_url.build_subscription_url("surge")?)
        .await?;
    let mut profile = SurgeProfile::new(raw_profile);
    profile.parse(convertor_url.build_convertor_url("surge")?);
    Ok(profile.to_string())
}

async fn rule_set_impl(
    state: Arc<AppState>,
    convertor_url: UrlBuilder,
    policies: Option<String>,
    boslife: Option<bool>,
) -> Result<String> {
    let raw_profile = state
        .service
        .get_raw_profile(convertor_url.build_subscription_url("surge")?)
        .await?;
    let profile = SurgeProfile::new(raw_profile);
    if let Some(policies) = policies {
        Ok(profile.extract_rule(policies))
    } else if let Some(true) = boslife {
        // 专门处理 boslife 订阅 url 的规则
        Ok(profile.extract_boslife_subscription())
    } else {
        Err(eyre!("Invalid query: missing policies or boslife flag"))
    }
}

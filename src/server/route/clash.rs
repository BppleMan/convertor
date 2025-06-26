use crate::config::url_builder::UrlBuilder;
use crate::error::AppError;
use crate::profile::clash_profile::ClashProfile;
use crate::profile::Profile;
use crate::server::route::AppState;
use crate::service::service_api::ServiceApi;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::Request;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashQuery {
    pub raw_url: String,
    #[serde(default)]
    pub policies: Option<String>,
    #[serde(default)]
    pub boslife: Option<bool>,
}

pub async fn profile(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(
        &request,
        &state.service.config.secret,
    )?;
    profile_impl(state, url_builder).await.map_err(Into::into)
}

pub async fn proxy_provider(
    State(state): State<Arc<AppState>>,
    query: Query<ClashQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(
        &request,
        &state.service.config.secret,
    )?;
    proxy_provider_impl(state, url_builder)
        .await
        .map_err(Into::into)
}

pub async fn rule_set(
    State(state): State<Arc<AppState>>,
    query: Query<ClashQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(
        &request,
        &state.service.config.secret,
    )?;
    rule_set_impl(state, url_builder).await.map_err(Into::into)
}

async fn profile_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
) -> Result<String> {
    let fixed_profile = ClashProfile::generate_profile(&url_builder)?;
    Ok(fixed_profile)
}

async fn proxy_provider_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
) -> Result<String> {
    let raw_profile = state
        .service
        .get_raw_profile(url_builder.build_subscription_url("clash")?)
        .await?;
    let mut profile = ClashProfile::from_str(&raw_profile)?;
    profile.organize_proxy_group();
    Ok(profile.to_string())
}

async fn rule_set_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
) -> Result<String> {
    let raw_profile = state
        .service
        .get_raw_profile(url_builder.build_subscription_url("clash")?)
        .await?;
    let mut profile = ClashProfile::from_str(&raw_profile)?;
    todo!();
    // profile.organize_proxy_group();
    Ok(profile.to_string())
}

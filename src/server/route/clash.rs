use crate::convertor_url::ConvertorUrl;
use crate::error::AppError;
use crate::profile::clash_profile::ClashProfile;
use crate::server::route::AppState;
use crate::service::service_api::ServiceApi;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::Request;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashQuery {
    pub base_url: String,
    pub token: String,
}

pub async fn profile(
    State(state): State<Arc<AppState>>,
    query: Query<ClashQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let convertor_url = ConvertorUrl::decode_from_request(
        &request,
        &state.service.config.secret,
    )?;
    profile_impl(state, query, convertor_url)
        .await
        .map_err(Into::into)
}

async fn profile_impl(
    state: Arc<AppState>,
    _query: Query<ClashQuery>,
    convertor_url: ConvertorUrl,
) -> Result<String> {
    let raw_profile = state
        .service
        .get_raw_profile(convertor_url.build_subscription_url("clash")?)
        .await?;
    let mut profile = ClashProfile::from_str(raw_profile)?;
    profile.organize_proxy_group();
    Ok(profile.to_string())
}

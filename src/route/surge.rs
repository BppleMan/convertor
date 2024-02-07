use axum::extract::Query;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::AppError;
use crate::profile::surge_profile::SurgeProfile;
use crate::route::get_raw_profile;

#[derive(Debug, Serialize, Deserialize)]
pub struct SurgeQuery {
    pub url: String,
    pub flag: String,
    #[serde(default)]
    pub policies: Option<String>,
}

pub async fn profile(query: Query<SurgeQuery>) -> Result<String, AppError> {
    info!("{:#?}", query.0);
    profile_impl(query).await.map_err(Into::into)
}

/// policy:
/// - Direct
/// - BosLife
/// - no-resolve
/// - force-remote-dns
pub async fn rule_set(query: Query<SurgeQuery>) -> Result<String, AppError> {
    info!("{:#?}", query.0);
    rule_set_impl(query).await.map_err(Into::into)
}

async fn profile_impl(query: Query<SurgeQuery>) -> Result<String> {
    let raw_profile = get_raw_profile(&query.url, &query.flag).await?;
    let mut profile = SurgeProfile::new(raw_profile);
    profile.replace_header();
    profile.organize_proxy_group();
    Ok(profile.to_string())
}

async fn rule_set_impl(query: Query<SurgeQuery>) -> Result<String> {
    let raw_profile = get_raw_profile(&query.url, &query.flag).await?;
    let profile = SurgeProfile::new(raw_profile);
    let policies = query.0.policies.as_ref().unwrap().split('|').collect::<Vec<_>>();
    Ok(profile.extract_rule(&policies))
}

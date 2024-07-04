use axum::body::Body;
use axum::extract::Query;
use axum::http::{header, HeaderValue, Request};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::AppError;
use crate::profile::get_raw_profile;
use crate::profile::surge_profile::SurgeProfile;

#[derive(Debug, Serialize, Deserialize)]
pub struct SurgeQuery {
    pub url: String,
    #[serde(default)]
    pub policies: Option<String>,
    #[serde(default)]
    pub boslife: Option<bool>,
}

pub async fn profile(
    query: Query<SurgeQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    info!("{:#?}", query.0);
    profile_impl(query, request.headers().get(header::HOST).cloned())
        .await
        .map_err(Into::into)
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

async fn profile_impl(
    query: Query<SurgeQuery>,
    host: Option<HeaderValue>,
) -> Result<String> {
    let raw_profile = get_raw_profile(&query.url, "surge").await?;
    let mut profile = SurgeProfile::new(raw_profile);
    let host = host
        .map(|h| h.to_str().map(|s| s.to_string()))
        .transpose()?;
    profile.replace_header(host, &query.url);
    profile.organize_proxy_group();
    Ok(profile.to_string())
}

async fn rule_set_impl(query: Query<SurgeQuery>) -> Result<String> {
    let raw_profile = get_raw_profile(&query.url, "surge").await?;
    let profile = SurgeProfile::new(raw_profile);
    if let Some(policies) = query.0.policies.as_ref() {
        let policies = policies.split('|').collect::<Vec<_>>();
        Ok(profile.extract_rule(&policies))
    } else if query.0.boslife == Some(true) {
        Ok(profile.extract_boslife_subscription())
    } else {
        Err(eyre!("Invalid query: missing policies or boslife flag"))
    }
}

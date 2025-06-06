use crate::convertor_url::ConvertorUrl;
use crate::error::AppError;
use crate::profile::get_raw_profile;
use crate::profile::surge_profile::SurgeProfile;
use crate::route::extract_convertor_url;
use axum::body::Body;
use axum::extract::Query;
use axum::http::Request;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SurgeQuery {
    pub base_url: String,
    pub token: String,
    #[serde(default)]
    pub policies: Option<String>,
    #[serde(default)]
    pub boslife: Option<bool>,
}

pub async fn profile(
    query: Query<SurgeQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let convertor_url = extract_convertor_url(&request)?;
    profile_impl(query, convertor_url).await.map_err(Into::into)
}

/// policy:
/// - Direct
/// - BosLife
/// - no-resolve
/// - force-remote-dns
pub async fn rule_set(
    query: Query<SurgeQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let convertor_url = extract_convertor_url(&request)?;
    rule_set_impl(query, convertor_url)
        .await
        .map_err(Into::into)
}

async fn profile_impl(
    _query: Query<SurgeQuery>,
    convertor_url: ConvertorUrl,
) -> Result<String> {
    let raw_profile =
        get_raw_profile(convertor_url.build_service_url("surge")?).await?;
    let mut profile = SurgeProfile::new(raw_profile);
    profile.parse(convertor_url.encode_to_convertor_url()?);
    Ok(profile.to_string())
}

async fn rule_set_impl(
    query: Query<SurgeQuery>,
    convertor_url: ConvertorUrl,
) -> Result<String> {
    let raw_profile =
        get_raw_profile(convertor_url.build_service_url("surge")?).await?;
    let profile = SurgeProfile::new(raw_profile);
    if let Some(policies) = query.policies.as_ref() {
        let policies = policies.split('|').collect::<Vec<_>>();
        Ok(profile.extract_rule(&policies))
    } else if query.boslife == Some(true) {
        // 专门处理 boslife 订阅 url 的规则
        Ok(profile.extract_boslife_subscription())
    } else {
        Err(eyre!("Invalid query: missing policies or boslife flag"))
    }
}

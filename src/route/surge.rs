use crate::airport::convertor_url::ConvertorUrl;
use crate::error::AppError;
use crate::profile::get_raw_profile;
use crate::profile::surge_profile::SurgeProfile;
use axum::body::Body;
use axum::extract::Query;
use axum::http::{header, HeaderValue, Request};
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::{Report, Result};
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
    let host = request
        .headers()
        .get(header::HOST)
        .ok_or(AppError(eyre!("Missing Host header")))?
        .to_str()
        .map_err(|e| AppError(eyre!(e)))?;
    let full_url = format!("http://{}{}", host, request.uri());
    let convertor_url = ConvertorUrl::decode_from_convertor_url(full_url)?;
    println!("{:?}", convertor_url);
    profile_impl(query).await.map_err(Into::into)
    Ok(format!("{:?}", convertor_url))
}

/// policy:
/// - Direct
/// - BosLife
/// - no-resolve
/// - force-remote-dns
pub async fn rule_set(query: Query<SurgeQuery>) -> Result<String, AppError> {
    rule_set_impl(query).await.map_err(Into::into)
}

async fn profile_impl(query: Query<SurgeQuery>, convertor_url: ConvertorUrl) -> Result<String> {
    let raw_profile = get_raw_profile(
        urlencoding::decode(&query.base_url)?,
        urlencoding::decode(&query.token)?,
        "surge",
    )
    .await?;
    let mut profile = SurgeProfile::new(raw_profile);
    // profile.parse(host, &query.base_url, &query.token);
    Ok(profile.to_string())
}

async fn rule_set_impl(query: Query<SurgeQuery>) -> Result<String> {
    let raw_profile = get_raw_profile(
        urlencoding::decode(&query.base_url)?,
        urlencoding::decode(&query.token)?,
        "surge",
    )
    .await?;
    let profile = SurgeProfile::new(raw_profile);
    if let Some(policies) = query.0.policies.as_ref() {
        let policies = policies.split('|').collect::<Vec<_>>();
        Ok(profile.extract_rule(&policies))
    } else if query.0.boslife == Some(true) {
        // 专门处理 boslife 订阅 url 的规则
        Ok(profile.extract_boslife_subscription())
    } else {
        Err(eyre!("Invalid query: missing policies or boslife flag"))
    }
}

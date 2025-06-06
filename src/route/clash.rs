use crate::convertor_url::ConvertorUrl;
use crate::error::AppError;
use crate::profile::clash_profile::ClashProfile;
use crate::profile::get_raw_profile;
use crate::route::extract_convertor_url;
use axum::body::Body;
use axum::extract::Query;
use axum::http::Request;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashQuery {
    pub base_url: String,
    pub token: String,
}

pub async fn profile(
    query: Query<ClashQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let convertor_url = extract_convertor_url(&request)?;
    profile_impl(query, convertor_url).await.map_err(Into::into)
}

async fn profile_impl(
    _query: Query<ClashQuery>,
    convertor_url: ConvertorUrl,
) -> Result<String> {
    let raw_profile =
        get_raw_profile(convertor_url.build_service_url("clash")?).await?;
    let mut profile = ClashProfile::from_str(raw_profile)?;
    profile.organize_proxy_group();
    Ok(profile.to_string())
}

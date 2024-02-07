use crate::error::AppError;
use crate::profile::clash_profile::ClashProfile;
use crate::route::get_raw_profile;
use axum::extract::Query;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashQuery {
    pub url: String,
    pub flag: String,
}

pub async fn profile(query: Query<ClashQuery>) -> Result<String, AppError> {
    info!("{:#?}", query.0);
    profile_impl(query).await.map_err(Into::into)
}

async fn profile_impl(query: Query<ClashQuery>) -> Result<String> {
    let raw_profile = get_raw_profile(&query.url, &query.flag).await?;
    let mut profile = ClashProfile::from_str(raw_profile)?;
    profile.organize_proxy_group();
    Ok(profile.to_string())
}

use crate::error::AppError;
use anyhow::{anyhow, Result};
use axum::extract::Query;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ini::Ini;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct SurgeQuery {
    pub url: String,
    pub flag: String,
}

pub async fn profile(query: Query<SurgeQuery>) -> Result<String, AppError> {
    info!("{}", query.url);
    profile_impl(query).await.map_err(Into::into)
}

pub async fn rule_set() {}

async fn profile_impl(query: Query<SurgeQuery>) -> Result<String> {
    let raw_profile = get_raw_profile(&query.0).await?;
    let ini = Ini::load_from_str(&raw_profile)?;
    ini.sections().for_each(|sec| println!("{:?}", sec));
    Ok(raw_profile)
}

async fn get_raw_profile(query: &SurgeQuery) -> Result<String> {
    let mut url = Url::from_str(&query.url)?;
    url.query_pairs_mut().append_pair("flag", &query.flag);
    reqwest::get(url).await?.text().await.map_err(Into::into)
}

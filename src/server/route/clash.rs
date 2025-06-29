use crate::error::AppError;
use crate::profile::clash_profile::ClashProfile;
use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::server::route::AppState;
use crate::subscription::url_builder::UrlBuilder;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::Request;
use color_eyre::eyre::eyre;
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

pub async fn profile(State(state): State<Arc<AppState>>, request: Request<Body>) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(&request, &state.convertor_config.secret)?;
    profile_impl(state, url_builder).await.map_err(Into::into)
}

pub async fn rule_set(
    State(state): State<Arc<AppState>>,
    query: Query<ClashQuery>,
    request: Request<Body>,
) -> Result<String, AppError> {
    let url_builder = UrlBuilder::decode_from_request(&request, &state.convertor_config.secret)?;
    rule_set_impl(state, url_builder, query.policies.clone(), query.boslife)
        .await
        .map_err(Into::into)
}

async fn profile_impl(state: Arc<AppState>, url_builder: UrlBuilder) -> Result<String> {
    let raw_profile = state
        .subscription_api
        .get_raw_profile(url_builder.build_subscription_url("clash")?)
        .await?;
    let mut template = ClashProfile::template()?;
    template.generate_profile(
        raw_profile,
        url_builder
            .service_url
            .host_str()
            .ok_or(eyre!("错误的订阅 URL, 未能解析出 host"))?,
        &url_builder,
    )?;
    template.secret = state.convertor_config.secret.clone();
    Ok(template.serialize())
}

async fn rule_set_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
    policies: Option<String>,
    boslife: Option<bool>,
) -> Result<String> {
    let raw_profile = state
        .subscription_api
        .get_raw_profile(url_builder.build_subscription_url("clash")?)
        .await?;
    let raw_profile = ClashProfile::from_str(&raw_profile)?;
    let sub_host = url_builder
        .service_url
        .host_str()
        .ok_or(eyre!("错误的订阅 URL, 未能解析出 host"))?;
    if let Some(true) = boslife {
        raw_profile
            .generate_rule_provider(RuleSetPolicy::BosLifeSubscription, sub_host)
            .ok_or_else(|| eyre!("未能找到匹配策略的规则以生成 Rule Provider"))
    } else if let Some(policies) = policies {
        let rsp = RuleSetPolicy::from_str(&policies)?;
        raw_profile
            .generate_rule_provider(rsp, sub_host)
            .ok_or_else(|| eyre!("未能找到匹配策略的规则以生成 Rule Provider"))
    } else {
        Err(eyre!("未知的查询参数: 缺少 policies 或 boslife 标志"))
    }
}

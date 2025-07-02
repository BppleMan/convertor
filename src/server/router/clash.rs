use crate::profile::clash_profile::ClashProfile;
use crate::profile::core::policy::Policy;
use crate::profile::renderer::clash_renderer::ClashRenderer;
use crate::server::router::AppState;
use crate::subscription::url_builder::UrlBuilder;
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

pub(super) async fn profile_impl(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
    let mut template = ClashProfile::template()?;
    template.merge(raw_profile, &url_builder, &state.convertor_config.secret)?;
    let mut profile = String::new();
    ClashRenderer::render_profile(&mut profile, &template)?;
    Ok(profile)
}

pub(super) async fn rule_set_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let raw_profile = ClashProfile::from_str(&raw_profile)?;
    let sub_host = url_builder
        .service_url
        .host_str()
        .ok_or(eyre!("错误的订阅 URL, 未能解析出 host"))?;
    raw_profile.rules_for_provider(policy, sub_host)
}

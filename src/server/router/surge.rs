use crate::profile::core::policy::Policy;
use crate::profile::renderer::surge_renderer::SurgeRenderer;
use crate::profile::surge_profile::SurgeProfile;
use crate::server::router::AppState;
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub(super) async fn profile_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
) -> Result<String> {
    let mut profile = SurgeProfile::parse(raw_profile.clone())?;
    profile.optimize(url_builder)?;
    let output = SurgeRenderer::render_profile(&profile)?;
    Ok(output)
}

pub(super) async fn rule_set_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = SurgeProfile::parse(raw_profile)?;
    let sub_host = url_builder
        .service_url
        .host_str()
        .ok_or(eyre!("错误的订阅 URL, 未能解析出 host"))?;
    let rules = profile.rules_for_provider(policy, sub_host);
    let output = SurgeRenderer::render_rules(&rules)?;
    Ok(output)
}

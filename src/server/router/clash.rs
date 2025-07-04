use crate::profile::clash_profile::ClashProfile;
use crate::profile::core::policy::Policy;
use crate::profile::renderer::clash_renderer::ClashRenderer;
use crate::server::router::AppState;
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashQuery {
    pub raw_url: String,
    #[serde(default)]
    pub policies: Option<String>,
    #[serde(default)]
    pub boslife: Option<bool>,
}

#[instrument(skip_all)]
pub(super) async fn profile_impl(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
    let mut template = ClashProfile::template()?;
    template.optimize(&url_builder, raw_profile, &state.config.secret)?;
    Ok(ClashRenderer::render_profile(&template)?)
}

#[instrument(skip_all)]
pub(super) async fn rule_set_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let raw_profile = ClashProfile::from_str(&raw_profile)?;
    let rules = raw_profile.rules_for_provider(policy, url_builder.sub_host()?)?;
    Ok(ClashRenderer::render_rules(&rules)?)
}

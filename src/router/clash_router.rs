use crate::profile::core::clash_profile::ClashProfile;
use crate::profile::core::policy::Policy;
use crate::profile::core::profile::Profile;
use crate::profile::renderer::Renderer;
use crate::profile::renderer::clash_renderer::ClashRenderer;
use crate::router::AppState;
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
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
pub(in crate::router) async fn profile_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
) -> Result<String> {
    let mut template = ClashProfile::template()?;
    template.optimize(&url_builder, Some(raw_profile), Some(&state.config.secret))?;
    Ok(ClashRenderer::render_profile(&template)?)
}

#[instrument(skip_all)]
pub(in crate::router) async fn rule_provider_impl(raw_profile: String, policy: Policy) -> Result<String> {
    let profile = ClashProfile::parse(raw_profile)?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
    }
}

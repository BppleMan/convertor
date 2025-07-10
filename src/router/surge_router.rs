use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::url_builder::UrlBuilder;
use color_eyre::Result;
use tracing::instrument;

#[instrument(skip_all)]
pub(super) async fn profile_impl(url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
    let mut profile = SurgeProfile::parse(raw_profile.clone())?;
    profile.optimize(&url_builder, None, Option::<&str>::None)?;
    Ok(SurgeRenderer::render_profile(&profile)?)
}

pub(super) async fn rule_provider_impl(url_builder: UrlBuilder, raw_profile: String, policy: Policy) -> Result<String> {
    let mut profile = SurgeProfile::parse(raw_profile)?;
    profile.optimize_rules(&url_builder)?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
    }
}

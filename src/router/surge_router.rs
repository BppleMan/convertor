use crate::core::error::ParseError;
use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::router::AppState;
use crate::url_builder::UrlBuilder;
use color_eyre::Result;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub(super) async fn profile_impl(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
    let profile = try_get_profile(state, url_builder, raw_profile).await?;
    Ok(SurgeRenderer::render_profile(&profile)?)
}

pub(super) async fn rule_provider_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, url_builder, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<SurgeProfile> {
    let profile = state
        .surge_cache
        .try_get_with(url_builder.clone(), async {
            let mut profile = SurgeProfile::parse(raw_profile.clone()).map_err(Arc::new)?;
            profile.optimize(&url_builder).map_err(Arc::new)?;
            Ok::<_, Arc<ParseError>>(profile)
        })
        .await?;
    Ok(profile)
}

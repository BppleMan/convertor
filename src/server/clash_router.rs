use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::query::convertor_query::ConvertorQuery;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::url_builder::UrlBuilder;
use crate::server::AppState;
use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn profile_impl(state: Arc<AppState>, query: ConvertorQuery, raw_profile: String) -> Result<String> {
    let profile = try_get_profile(state, query, raw_profile).await?;
    Ok(ClashRenderer::render_profile(&profile)?)
}

#[instrument(skip_all)]
pub async fn rule_provider_impl(
    state: Arc<AppState>,
    query: ConvertorQuery,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, query, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, query: ConvertorQuery, raw_profile: String) -> Result<ClashProfile> {
    let profile = state
        .clash_cache
        .try_get_with(query.clone(), async {
            let profile = ClashProfile::parse(raw_profile)?;
            let url_builder = UrlBuilder::from_query(&state.config.secret, query)?;
            let mut template = ClashProfile::template()?;
            template.patch(profile)?;
            template.convert(&url_builder)?;
            Ok::<_, Report>(template)
        })
        .await
        .map_err(|e| eyre!(e))?;
    Ok(profile)
}

use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::query::convertor_query::ConvertorQuery;
use crate::core::query::rule_provider_query::RuleProviderQuery;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::url_builder::UrlBuilder;
use crate::server::{AppState, ProfileCacheKey};
use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn profile_impl(state: Arc<AppState>, query: ConvertorQuery, raw_profile: String) -> Result<String> {
    let cache_key = query.cache_key();
    let url_builder = UrlBuilder::from_convertor_query(&state.config.secret, query)?;
    let profile = try_get_profile(state, cache_key, &url_builder, raw_profile).await?;
    Ok(ClashRenderer::render_profile(&profile)?)
}

#[instrument(skip_all)]
pub async fn rule_provider_impl(
    state: Arc<AppState>,
    query: RuleProviderQuery,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let cache_key = query.cache_key();
    let url_builder = UrlBuilder::from_rule_provider_query(&state.config.secret, query)?;
    let profile = try_get_profile(state, cache_key, &url_builder, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(
    state: Arc<AppState>,
    cache_key: ProfileCacheKey,
    url_builder: &UrlBuilder,
    raw_profile: String,
) -> Result<ClashProfile> {
    let profile = state
        .clash_cache
        .try_get_with(cache_key, async {
            let profile = ClashProfile::parse(raw_profile)?;
            let mut template = ClashProfile::template()?;
            template.patch(profile)?;
            template.convert(&url_builder)?;
            Ok::<_, Report>(template)
        })
        .await
        .map_err(|e| eyre!(e))?;
    Ok(profile)
}

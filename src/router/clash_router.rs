use crate::core::error::ParseError;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::router::AppState;
use crate::url_builder::UrlBuilder;
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
    let profile = try_get_profile(state, url_builder, raw_profile).await?;
    Ok(ClashRenderer::render_profile(&profile)?)
}

#[instrument(skip_all)]
pub(in crate::router) async fn rule_provider_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, url_builder, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<ClashProfile> {
    let profile = state
        .clash_cache
        .try_get_with(url_builder.clone(), async {
            let profile = ClashProfile::parse(raw_profile)?;
            let mut template = ClashProfile::template()?;
            template.merge(profile)?;
            template.optimize(&url_builder).map_err(Arc::new)?;
            Ok::<_, Arc<ParseError>>(template)
        })
        .await?;
    Ok(profile)
}

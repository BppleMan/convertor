use crate::server::AppState;
use color_eyre::Result;
use convertor_core::core::error::ParseError;
use convertor_core::core::profile::clash_profile::ClashProfile;
use convertor_core::core::profile::policy::Policy;
use convertor_core::core::profile::profile::Profile;
use convertor_core::core::renderer::Renderer;
use convertor_core::core::renderer::clash_renderer::ClashRenderer;
use convertor_core::url::ConvertorUrl;
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
pub async fn profile_impl(state: Arc<AppState>, url: ConvertorUrl, raw_profile: String) -> Result<String> {
    let profile = try_get_profile(state, url, raw_profile).await?;
    Ok(ClashRenderer::render_profile(&profile)?)
}

#[instrument(skip_all)]
pub async fn rule_provider_impl(
    state: Arc<AppState>,
    url: ConvertorUrl,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, url, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, url: ConvertorUrl, raw_profile: String) -> Result<ClashProfile> {
    let profile = state
        .clash_cache
        .try_get_with(url.clone(), async {
            let profile = ClashProfile::parse(raw_profile)?;
            let mut template = ClashProfile::template()?;
            template.merge(profile)?;
            template.optimize(&url).map_err(Arc::new)?;
            Ok::<_, Arc<ParseError>>(template)
        })
        .await?;
    Ok(profile)
}

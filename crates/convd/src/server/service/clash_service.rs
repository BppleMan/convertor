use color_eyre::Report;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use convertor::config::ConvertorConfig;
use convertor::core::profile::Profile;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::policy::Policy;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::url::url_builder::UrlBuilder;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct ClashService {
    pub config: Arc<ConvertorConfig>,
    pub profile_cache: Cache<UrlBuilder, ClashProfile>,
}

impl ClashService {
    pub fn new(config: Arc<ConvertorConfig>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        Ok(ClashRenderer::render_profile(&profile)?)
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, url_builder: UrlBuilder, raw_profile: String, policy: Policy) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        match profile.get_provider_rules_with_policy(&policy) {
            None => Ok(String::new()),
            Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
        }
    }

    async fn try_get_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<ClashProfile> {
        self.profile_cache
            .try_get_with(url_builder.clone(), async {
                let profile = ClashProfile::parse(raw_profile)?;
                let mut template = ClashProfile::template()?;
                template.patch(profile)?;
                template.convert(&url_builder)?;
                Ok::<_, Report>(template)
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

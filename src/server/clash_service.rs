use crate::common::config::ConvertorConfig;
use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::url_builder::UrlBuilder;
use crate::server::app_state::ProfileCacheKey;
use crate::server::query::profile_query::ProfileQuery;
use crate::server::query::rule_provider_query::RuleProviderQuery;
use color_eyre::Report;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

pub struct ClashService {
    pub config: Arc<ConvertorConfig>,
    pub profile_cache: Cache<ProfileCacheKey, ClashProfile>,
}

impl ClashService {
    pub fn new(config: Arc<ConvertorConfig>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, query: ProfileQuery, raw_profile: String) -> Result<String> {
        let cache_key = query.cache_key();
        let url_builder = UrlBuilder::from_convertor_query(&self.config.secret, query)?;
        let profile = self.try_get_profile(cache_key, &url_builder, raw_profile).await?;
        Ok(ClashRenderer::render_profile(&profile)?)
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, query: RuleProviderQuery, raw_profile: String) -> Result<String> {
        let cache_key = query.cache_key();
        let url_builder = UrlBuilder::from_rule_provider_query(&self.config.secret, &query)?;
        let profile = self.try_get_profile(cache_key, &url_builder, raw_profile).await?;
        match profile.get_provider_rules_with_policy(&query.policy.into()) {
            None => Ok(String::new()),
            Some(provider_rules) => Ok(ClashRenderer::render_provider_rules(provider_rules)?),
        }
    }

    async fn try_get_profile(
        &self,
        cache_key: ProfileCacheKey,
        url_builder: &UrlBuilder,
        raw_profile: String,
    ) -> Result<ClashProfile> {
        self.profile_cache
            .try_get_with(cache_key, async {
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

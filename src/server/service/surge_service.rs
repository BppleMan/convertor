use crate::common::config::ConvertorConfig;
use crate::core::convertor_url::ConvertorUrlType;
use crate::core::profile::Profile;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::url_builder::UrlBuilder;
use crate::provider_api::ProviderApi;
use crate::provider_api::boslife_log::BosLifeLogs;
use crate::server::error::AppError;
use crate::server::query::ConvertorQuery;
use color_eyre::Report;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct SurgeService {
    pub config: Arc<ConvertorConfig>,
    pub profile_cache: Cache<ConvertorQuery, SurgeProfile>,
}

impl SurgeService {
    pub fn new(config: Arc<ConvertorConfig>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, query: ConvertorQuery, raw_profile: String) -> Result<String> {
        let url_builder = UrlBuilder::from_convertor_query(query.clone(), &self.config.secret)?;
        let profile = self.try_get_profile(query, &url_builder, raw_profile).await?;
        Ok(SurgeRenderer::render_profile(&profile)?)
    }

    #[instrument(skip_all)]
    pub async fn raw_profile(&self, query: ConvertorQuery, raw_profile: String) -> Result<String> {
        let url_builder = UrlBuilder::from_convertor_query(query, &self.config.secret)?;
        let surge_header = url_builder.build_surge_header(ConvertorUrlType::RawProfile)?;
        let (_, right) = raw_profile
            .split_once('\n')
            .ok_or(eyre!("错误的原始配置, 未能找出第一行: {raw_profile}"))?;
        Ok(format!("{}\n{}", surge_header, right))
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, query: ConvertorQuery, raw_profile: String) -> Result<String> {
        let policy = query.policy.as_ref().unwrap().clone();
        let url_builder = UrlBuilder::from_convertor_query(query.clone(), &self.config.secret)?;
        let profile = self.try_get_profile(query, &url_builder, raw_profile).await?;
        match profile.get_provider_rules_with_policy(&policy) {
            None => Ok(String::new()),
            Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
        }
    }

    #[instrument(skip_all)]
    pub async fn sub_logs(&self, api: &ProviderApi) -> Result<BosLifeLogs, AppError> {
        Ok(api.get_sub_logs().await?)
    }

    async fn try_get_profile(
        &self,
        query: ConvertorQuery,
        url_builder: &UrlBuilder,
        raw_profile: String,
    ) -> Result<SurgeProfile> {
        self.profile_cache
            .try_get_with(query, async {
                let mut profile = SurgeProfile::parse(raw_profile.clone())?;
                profile.convert(&url_builder)?;
                Ok::<_, Report>(profile)
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

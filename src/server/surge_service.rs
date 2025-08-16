use crate::api::SubProviderWrapper;
use crate::api::boslife_sub_log::BosLifeSubLog;
use crate::common::config::ConvertorConfig;
use crate::core::profile::Profile;
use crate::core::profile::surge_header::SurgeHeaderType;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::url_builder::UrlBuilder;
use crate::server::app_state::ProfileCacheKey;
use crate::server::error::AppError;
use crate::server::query::profile_query::ProfileQuery;
use crate::server::query::rule_provider_query::RuleProviderQuery;
use crate::server::query::sub_logs_query::SubLogsQuery;
use color_eyre::Report;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct SurgeService {
    pub config: Arc<ConvertorConfig>,
    pub profile_cache: Cache<ProfileCacheKey, SurgeProfile>,
}

impl SurgeService {
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
        Ok(SurgeRenderer::render_profile(&profile)?)
    }

    #[instrument(skip_all)]
    pub async fn raw_profile(&self, query: ProfileQuery, raw_profile: String) -> Result<String> {
        let url_builder = UrlBuilder::from_convertor_query(&self.config.secret, query)?;
        let surge_header = url_builder.build_managed_config_header(SurgeHeaderType::RawProfile);
        let (_, right) = raw_profile
            .split_once('\n')
            .ok_or(eyre!("错误的原始配置, 未能找出第一行: {raw_profile}"))?;
        Ok(format!("{}\n{}", surge_header, right))
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, query: RuleProviderQuery, raw_profile: String) -> Result<String> {
        let url_builder = UrlBuilder::from_rule_provider_query(&self.config.secret, &query)?;
        let profile = self
            .try_get_profile(query.cache_key(), &url_builder, raw_profile)
            .await?;
        match profile.get_provider_rules_with_policy(&query.policy.into()) {
            None => Ok(String::new()),
            Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
        }
    }

    #[instrument(skip_all)]
    pub async fn sub_logs(
        &self,
        query: SubLogsQuery,
        api: &SubProviderWrapper,
    ) -> Result<Vec<BosLifeSubLog>, AppError> {
        if query.secret != self.config.secret {
            Err(AppError::Unauthorized("Invalid secret".to_string()))
        } else {
            let logs = api.get_sub_logs().await?;
            let start = (query.page - 1) * query.page_size;
            let logs: Vec<BosLifeSubLog> = logs.0.into_iter().skip(start).take(query.page_size).collect();
            Ok(logs)
        }
    }

    async fn try_get_profile(
        &self,
        cache_key: ProfileCacheKey,
        url_builder: &UrlBuilder,
        raw_profile: String,
    ) -> Result<SurgeProfile> {
        self.profile_cache
            .try_get_with(cache_key, async {
                let mut profile = SurgeProfile::parse(raw_profile.clone())?;
                profile.convert(&url_builder)?;
                Ok::<_, Report>(profile)
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

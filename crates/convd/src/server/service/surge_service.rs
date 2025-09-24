use color_eyre::Report;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use convertor::config::Config;
use convertor::core::profile::Profile;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use convertor::url::convertor_url::UrlType;
use convertor::url::url_builder::UrlBuilder;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct SurgeService {
    pub config: Arc<Config>,
    pub profile_cache: Cache<UrlBuilder, SurgeProfile>,
}

impl SurgeService {
    pub fn new(config: Arc<Config>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        Ok(SurgeRenderer::render_profile(&profile)?)
    }

    #[instrument(skip_all)]
    pub async fn raw_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
        let surge_header = url_builder.build_surge_header(UrlType::RawProfile)?;
        let (_, right) = raw_profile
            .split_once('\n')
            .ok_or(eyre!("错误的原始配置, 未能找出第一行: {raw_profile}"))?;
        Ok(format!("{surge_header}\n{right}"))
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, url_builder: UrlBuilder, raw_profile: String, policy: Policy) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        match profile.get_provider_rules_with_policy(&policy) {
            None => Ok(String::new()),
            Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
        }
    }

    pub async fn try_get_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<SurgeProfile> {
        self.profile_cache
            .try_get_with(url_builder.clone(), async {
                let mut profile = SurgeProfile::parse(raw_profile.clone())?;
                profile.convert(&url_builder)?;
                Ok::<_, Report>(profile)
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

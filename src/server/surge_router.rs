use crate::api::boslife_sub_log::BosLifeSubLog;
use crate::core::profile::Profile;
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::query::convertor_query::ConvertorQuery;
use crate::core::query::rule_provider_query::RuleProviderQuery;
use crate::core::query::sub_logs_query::SubLogsQuery;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::url_builder::UrlBuilder;
use crate::server::error::AppError;
use crate::server::{AppState, ProfileCacheKey};
use axum::Json;
use axum::extract::{RawQuery, State};
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn profile_impl(state: Arc<AppState>, query: ConvertorQuery, raw_profile: String) -> Result<String> {
    let cache_key = query.cache_key();
    let url_builder = UrlBuilder::from_convertor_query(&state.config.secret, query)?;
    let profile = try_get_profile(state, cache_key, &url_builder, raw_profile).await?;
    Ok(SurgeRenderer::render_profile(&profile)?)
}

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
        Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
    }
}

pub async fn sub_logs(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<BosLifeSubLog>>, AppError> {
    let query = query.as_ref().ok_or_eyre(eyre!("订阅记录必须传递参数"))?;
    let sub_log_query = SubLogsQuery::decode_from_query_string(query, &state.config.secret)?;
    let provider = sub_log_query.provider;
    if sub_log_query.secret != state.config.secret {
        return Err(AppError::Unauthorized("Invalid secret".to_string()));
    }
    let Some(api) = state.api_map.get(&provider) else {
        return Err(AppError::NoSubProvider);
    };
    let logs = api.get_sub_logs().await?;
    let start = (sub_log_query.page - 1) * sub_log_query.page_size;
    let logs = logs.0.into_iter().skip(start).take(sub_log_query.page_size).collect();
    Ok(Json(logs))
}

async fn try_get_profile(
    state: Arc<AppState>,
    cache_key: ProfileCacheKey,
    url_builder: &UrlBuilder,
    raw_profile: String,
) -> Result<SurgeProfile> {
    state
        .surge_cache
        .try_get_with(cache_key, async {
            let mut profile = SurgeProfile::parse(raw_profile.clone())?;
            profile.convert(&url_builder)?;
            Ok::<_, Report>(profile)
        })
        .await
        .map_err(|e| eyre!(e))
}

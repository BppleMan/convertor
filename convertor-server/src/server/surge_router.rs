use crate::server::{AppError, AppState};
use axum::Json;
use axum::extract::{RawQuery, State};
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::eyre;
use convertor_core::api::boslife_sub_log::BosLifeSubLog;
use convertor_core::core::error::ParseError;
use convertor_core::core::profile::Profile;
use convertor_core::core::profile::policy::Policy;
use convertor_core::core::profile::surge_profile::SurgeProfile;
use convertor_core::core::renderer::Renderer;
use convertor_core::core::renderer::surge_renderer::SurgeRenderer;
use convertor_core::router::query::SubLogQuery;
use convertor_core::url::ConvertorUrl;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn profile_impl(state: Arc<AppState>, url: ConvertorUrl, raw_profile: String) -> Result<String> {
    let profile = try_get_profile(state, url, raw_profile).await?;
    Ok(SurgeRenderer::render_profile(&profile)?)
}

pub async fn rule_provider_impl(
    state: Arc<AppState>,
    url: ConvertorUrl,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, url, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, url: ConvertorUrl, raw_profile: String) -> Result<SurgeProfile> {
    let profile = state
        .surge_cache
        .try_get_with(url.clone(), async {
            let mut profile = SurgeProfile::parse(raw_profile.clone()).map_err(Arc::new)?;
            profile.optimize(&url).map_err(Arc::new)?;
            Ok::<_, Arc<ParseError>>(profile)
        })
        .await?;
    Ok(profile)
}

pub async fn sub_logs(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<BosLifeSubLog>>, AppError> {
    let query = query.as_ref().ok_or_eyre(eyre!("订阅记录必须传递参数"))?;
    let mut sub_log_query = SubLogQuery::decode_from_query_string(query, &state.config.secret)?;
    if sub_log_query.secret != state.config.secret {
        return Err(AppError::Unauthorized("Invalid secret".to_string()));
    }
    let logs = state.api.get_sub_logs().await?;
    let logs = if let (Some(current), Some(size)) = (sub_log_query.page_current.take(), sub_log_query.page_size.take())
    {
        let start = (current - 1) * size;
        logs.0.into_iter().skip(start).take(size).collect()
    } else {
        logs.0
    };
    Ok(Json(logs))
}

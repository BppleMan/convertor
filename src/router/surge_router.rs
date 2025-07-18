use crate::core::error::ParseError;
use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::error::AppError;
use crate::router::AppState;
use crate::router::query::SubLogQuery;
use crate::service_provider::api::subscription_log::SubscriptionLog;
use crate::url_builder::UrlBuilder;
use axum::Json;
use axum::extract::{RawQuery, State};
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use color_eyre::eyre::eyre;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub(super) async fn profile_impl(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
    let profile = try_get_profile(state, url_builder, raw_profile).await?;
    Ok(SurgeRenderer::render_profile(&profile)?)
}

pub(super) async fn rule_provider_impl(
    state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Policy,
) -> Result<String> {
    let profile = try_get_profile(state, url_builder, raw_profile).await?;
    match profile.get_provider_rules_with_policy(&policy) {
        None => Ok(String::new()),
        Some(provider_rules) => Ok(SurgeRenderer::render_provider_rules(provider_rules)?),
    }
}

async fn try_get_profile(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<SurgeProfile> {
    let profile = state
        .surge_cache
        .try_get_with(url_builder.clone(), async {
            let mut profile = SurgeProfile::parse(raw_profile.clone()).map_err(Arc::new)?;
            profile.optimize(&url_builder).map_err(Arc::new)?;
            Ok::<_, Arc<ParseError>>(profile)
        })
        .await?;
    Ok(profile)
}

pub async fn sub_logs(
    State(state): State<Arc<AppState>>,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<SubscriptionLog>>, AppError> {
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

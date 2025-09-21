use crate::server::app_state::AppState;
use crate::server::response::ApiError;
use crate::server::router::{OptionalScheme, parse_query};
use axum::extract::{Path, RawQuery, State};
use axum_extra::TypedHeader;
use axum_extra::extract::Host;
use axum_extra::headers::UserAgent;
use convertor::config::client_config::ProxyClient;
use convertor::config::provider_config::Provider;
use convertor::url::url_builder::UrlBuilder;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn raw_profile(
    Path((client, provider)): Path<(ProxyClient, Provider)>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<String, ApiError> {
    let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_profile()?;
    let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client, provider)?;
    let api = state.api_map.get(&provider).ok_or_else(|| ApiError::NoSubProvider)?;
    let raw_profile = api.get_raw_profile(client, user_agent).await?;
    match client {
        ProxyClient::Surge => Ok(state.surge_service.raw_profile(url_builder, raw_profile).await?),
        ProxyClient::Clash => Err(ApiError::RawProfileUnsupportedClient(client)),
    }
}

#[instrument(skip_all)]
pub async fn profile(
    Path((client, provider)): Path<(ProxyClient, Provider)>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<String, ApiError> {
    let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_profile()?;
    let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client, provider)?;
    let api = state.api_map.get(&provider).ok_or_else(|| ApiError::NoSubProvider)?;
    let raw_profile = api.get_raw_profile(client, user_agent).await?;
    let profile = match client {
        ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
        ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
    }?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(
    Path((client, provider)): Path<(ProxyClient, Provider)>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> color_eyre::Result<String, ApiError> {
    let (query, policy) =
        parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_rule_provider()?;
    let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client, provider)?;
    let api = state.api_map.get(&provider).ok_or_else(|| ApiError::NoSubProvider)?;
    let raw_profile = api.get_raw_profile(client, user_agent).await?;
    let rules = match client {
        ProxyClient::Surge => {
            state
                .surge_service
                .rule_provider(url_builder, raw_profile, policy)
                .await
        }
        ProxyClient::Clash => {
            state
                .clash_service
                .rule_provider(url_builder, raw_profile, policy)
                .await
        }
    }?;
    Ok(rules)
}

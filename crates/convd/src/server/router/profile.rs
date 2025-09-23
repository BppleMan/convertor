use crate::server::app_state::AppState;
use crate::server::response::ApiError;
use crate::server::router::{OptionalScheme, parse_query};
use axum::extract::{Path, RawQuery, State};
use axum::http::HeaderMap;
use axum_extra::TypedHeader;
use axum_extra::extract::Host;
use axum_extra::headers::UserAgent;
use convertor::config::proxy_client::ProxyClient;
use convertor::config::subscription_config::Headers;
use convertor::url::url_builder::UrlBuilder;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn raw_profile(
    Path(client): Path<ProxyClient>,
    Host(host): Host,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    scheme: Option<OptionalScheme>,
    header_map: HeaderMap,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<String, ApiError> {
    let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_profile()?;
    let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client)?;
    let sub_url = url_builder.build_raw_url();
    let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers, user_agent);
    let raw_profile = state.provider.get_raw_profile(sub_url.into(), headers).await?;
    match client {
        ProxyClient::Surge => Ok(state.surge_service.raw_profile(url_builder, raw_profile).await?),
        ProxyClient::Clash => Err(ApiError::RawProfileUnsupportedClient(client)),
    }
}

#[instrument(skip_all)]
pub async fn profile(
    Path(client): Path<ProxyClient>,
    Host(host): Host,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    scheme: Option<OptionalScheme>,
    header_map: HeaderMap,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<String, ApiError> {
    let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_profile()?;
    let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client)?;
    let sub_url = url_builder.build_raw_url();
    let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers, user_agent);
    let raw_profile = state.provider.get_raw_profile(sub_url.into(), headers).await?;
    let profile = match client {
        ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
        ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
    }?;
    Ok(profile)
}

#[instrument(skip_all)]
pub async fn rule_provider(
    Path(client): Path<ProxyClient>,
    Host(host): Host,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    scheme: Option<OptionalScheme>,
    header_map: HeaderMap,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> color_eyre::Result<String, ApiError> {
    let (query, policy) =
        parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_rule_provider()?;
    let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client)?;
    let sub_url = url_builder.build_raw_url();
    let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers, user_agent);
    let raw_profile = state.provider.get_raw_profile(sub_url.into(), headers).await?;
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

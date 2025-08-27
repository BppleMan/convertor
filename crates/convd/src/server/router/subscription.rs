use crate::server::app_state::AppState;
use crate::server::error::AppError;
use crate::server::router::{OptionalScheme, parse_query};
use axum::Json;
use axum::extract::{Path, RawQuery, State};
use axum_extra::TypedHeader;
use axum_extra::extract::Host;
use axum_extra::headers::UserAgent;
use convertor::config::client_config::ProxyClient;
use convertor::config::provider_config::Provider;
use convertor::provider_api::BosLifeLogs;
use convertor::url::url_result::UrlResult;
use std::sync::Arc;

#[allow(unused)]
pub async fn subscription(
    Path((client, provider)): Path<(ProxyClient, Provider)>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<Json<UrlResult>, AppError> {
    // let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    // let server = Url::parse(format!("{scheme}://{host}").as_str()).wrap_err("解析请求 URL 失败");
    // ready(match server {
    //     Ok(server) => Ok(state.config.subscription_url(&server)),
    //     Err(e) => Err(AppError::from(e)),
    // })
    todo!()
}

pub async fn sub_logs(
    Path(provider): Path<Provider>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<Json<BosLifeLogs>, AppError> {
    parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_sub_logs(&state.config.secret)?;
    let api = state.api_map.get(&provider).ok_or_else(|| AppError::NoSubProvider)?;
    Ok(Json(api.get_sub_logs().await?))
}

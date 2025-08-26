mod actuator;

use crate::server::AppState;
use crate::server::error::AppError;
use axum::extract::{FromRequestParts, OptionalFromRequestParts, Path, RawQuery, State};
use axum::http::request::Parts;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::TypedHeader;
use axum_extra::extract::Host;
use axum_extra::headers::UserAgent;
use color_eyre::eyre::{WrapErr, eyre};
use convertor::config::client_config::ProxyClient;
use convertor::config::provider_config::Provider;
use convertor::provider_api::BosLifeLogs;
use convertor::url::query::ConvertorQuery;
use std::sync::Arc;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::instrument;
use url::Url;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Html(include_str!("../../assets/index.html")) }))
        .route("/healthy", get(|| async { "ok" }))
        .route("/redis", get(actuator::redis))
        .route("/version", get(actuator::version))
        .route("/raw-profile/{client}/{provider}", get(raw_profile))
        .route("/profile/{client}/{provider}", get(profile))
        .route("/rule-provider/{client}/{provider}", get(rule_provider))
        .route("/sub-logs/{provider}", get(sub_logs))
        .with_state(Arc::new(app_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
}

pub async fn raw_profile(
    Path((client, provider)): Path<(ProxyClient, Provider)>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<String, AppError> {
    let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    let server = Url::parse(format!("{scheme}://{host}").as_str()).wrap_err("解析请求 URL 失败")?;
    let query = query_string
        .map(|query_string| {
            ConvertorQuery::parse_from_query_string(query_string, &state.config.secret, server, client, provider)
        })
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    query.check_for_profile()?;
    let Some(api) = state.api_map.get(&provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(client, user_agent).await?;
    match client {
        ProxyClient::Surge => Ok(state.surge_service.raw_profile(query, raw_profile).await?),
        ProxyClient::Clash => Err(AppError::RawProfileUnsupportedClient(client)),
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
) -> Result<String, AppError> {
    let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    let server = Url::parse(format!("{scheme}://{host}").as_str()).wrap_err("解析请求 URL 失败")?;
    let query = query_string
        .map(|query_string| {
            ConvertorQuery::parse_from_query_string(query_string, &state.config.secret, server, client, provider)
        })
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    query.check_for_profile()?;
    let Some(api) = state.api_map.get(&provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(client, user_agent).await?;
    let profile = match client {
        ProxyClient::Surge => state.surge_service.profile(query, raw_profile).await,
        ProxyClient::Clash => state.clash_service.profile(query, raw_profile).await,
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
) -> color_eyre::Result<String, AppError> {
    let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    let server = Url::parse(format!("{scheme}://{host}").as_str()).wrap_err("解析请求 URL 失败")?;
    let query = query_string
        .map(|query_string| {
            ConvertorQuery::parse_from_query_string(query_string, &state.config.secret, server, client, provider)
        })
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    query.check_for_rule_provider()?;
    let Some(api) = state.api_map.get(&provider) else {
        return Err(AppError::NoSubProvider);
    };
    let raw_profile = api.get_raw_profile(client, user_agent).await?;
    let rules = match client {
        ProxyClient::Surge => state.surge_service.rule_provider(query, raw_profile).await,
        ProxyClient::Clash => state.clash_service.rule_provider(query, raw_profile).await,
    }?;
    Ok(rules)
}

pub async fn sub_logs(
    Path(provider): Path<Provider>,
    Host(host): Host,
    scheme: Option<OptionalScheme>,
    State(state): State<Arc<AppState>>,
    RawQuery(query_string): RawQuery,
) -> Result<Json<BosLifeLogs>, AppError> {
    let client = ProxyClient::Surge; // 订阅日志只支持 Surge 客户端
    let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    let server = Url::parse(format!("{scheme}://{host}").as_str()).wrap_err("解析请求 URL 失败")?;
    let query = query_string
        .map(|query_string| {
            ConvertorQuery::parse_from_query_string(query_string, &state.config.secret, server, client, provider)
        })
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    query.check_for_sub_logs()?;
    if query.secret.as_ref().unwrap() != &state.config.secret {
        return Err(AppError::Unauthorized("无效的密钥".to_string()));
    }
    let Some(api) = state.api_map.get(&provider) else {
        return Err(AppError::NoSubProvider);
    };
    Ok(Json(api.get_sub_logs().await?))
}

#[derive(Debug, Clone)]
pub struct OptionalScheme(pub String);

impl<S> OptionalFromRequestParts<S> for OptionalScheme
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Option<Self>, Self::Rejection>> + Send {
        async {
            let scheme = axum_extra::extract::Scheme::from_request_parts(parts, state).await;
            match scheme {
                Ok(scheme) => Ok(Some(OptionalScheme(scheme.0))),
                Err(_) => Ok(None),
            }
        }
    }
}

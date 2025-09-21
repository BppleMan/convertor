pub mod actuator;
pub mod angular;
pub mod api;
pub mod profile;

use crate::server::AppState;
use crate::server::layer::trace::convd_trace_layer;
use crate::server::response::ApiError;
use axum::Router;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum::response::Redirect;
use axum::routing::get;
use axum_extra::extract::Scheme;
use color_eyre::eyre::{WrapErr, eyre};
use convertor::url::query::ConvertorQuery;
use std::sync::Arc;
use url::Url;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/dashboard/") }))
        .route("/dashboard", get(|| async { Redirect::permanent("/dashboard/") }))
        .route("/index.html", get(|| async { Redirect::permanent("/dashboard/") }))
        .route("/actuator/healthy", get(actuator::healthy))
        .route("/actuator/ready", get(actuator::redis))
        .route("/actuator/redis", get(actuator::redis))
        .route("/actuator/version", get(actuator::version))
        .route("/raw-profile/{client}/{provider}", get(profile::raw_profile))
        .route("/profile/{client}/{provider}", get(profile::profile))
        .route("/rule-provider/{client}/{provider}", get(profile::rule_provider))
        .route(
            "/api/subscription/{client}/{provider}",
            get(api::subscription::subscription),
        )
        .route("/api/sub-logs/{provider}", get(api::subscription::sub_logs))
        .nest("/dashboard/", angular::router())
        .with_state(Arc::new(app_state))
        .layer(convd_trace_layer())
}

pub fn parse_query(
    app_state: &AppState,
    scheme: Option<OptionalScheme>,
    host: impl AsRef<str>,
    query_string: Option<impl AsRef<str>>,
) -> color_eyre::Result<ConvertorQuery> {
    let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    let host = host.as_ref();
    let server = Url::parse(format!("{scheme}://{host}").as_str()).wrap_err("解析请求 URL 失败")?;
    let query = query_string
        .map(|query_string| ConvertorQuery::parse_from_query_string(query_string, &app_state.config.secret, server))
        .ok_or_else(|| eyre!("查询参数不能为空"))?
        .wrap_err("解析查询字符串失败")?;
    Ok(query)
}

#[derive(Debug, Clone)]
pub struct OptionalScheme(pub String);

impl<S> OptionalFromRequestParts<S> for OptionalScheme
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Option<Self>, Self::Rejection> {
        let scheme = Scheme::from_request_parts(parts, state).await;
        match scheme {
            Ok(scheme) => Ok(Some(OptionalScheme(scheme.0))),
            Err(_) => Ok(None),
        }
    }
}

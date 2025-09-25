pub mod actuator;
pub mod angular;
pub mod api;
pub mod profile;

use crate::server::AppState;
use crate::server::layer::trace::convd_trace_layer;
use crate::server::response::{ApiError, AppError};
use axum::Router;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum::response::Redirect;
use axum::routing::get;
use axum_extra::extract::Scheme;
use color_eyre::eyre::{WrapErr, eyre};
use convertor::error::QueryError;
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
        .route("/raw/{client}", get(profile::raw_profile))
        .route("/profile/{client}", get(profile::profile))
        .route("/rule-provider/{client}", get(profile::rule_provider))
        .route("/api/subscription/{client}", get(api::subscription::subscription))
        .nest("/dashboard/", angular::router())
        .with_state(Arc::new(app_state))
        .layer(convd_trace_layer())
}

pub fn parse_query(
    app_state: &AppState,
    scheme: Option<OptionalScheme>,
    host: impl AsRef<str>,
    query_string: Option<impl AsRef<str>>,
) -> Result<ConvertorQuery, QueryError> {
    let scheme = scheme.map(|s| s.0).unwrap_or("http".to_string());
    let host = host.as_ref();
    let server = Url::parse(format!("{scheme}://{host}").as_str())?;
    let query = query_string
        .map(|query_string| ConvertorQuery::parse_from_query_string(query_string, &app_state.config.secret, server))
        .transpose()?
        .ok_or(QueryError::EmptyQuery)?;
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

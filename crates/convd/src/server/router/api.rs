pub mod subscription {
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
    use convertor::url::url_builder::UrlBuilder;
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
        let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?
            .check_for_subscription(&state.config.secret)?;
        let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client, provider)?;
        let api = state.api_map.get(&provider).ok_or_else(|| AppError::NoSubProvider)?;
        let raw_profile = api.get_raw_profile(client, user_agent).await?;
        let profile = match client {
            ProxyClient::Surge => state.surge_service.profile(url_builder, raw_profile).await,
            ProxyClient::Clash => state.clash_service.profile(url_builder, raw_profile).await,
        }?;
        Ok(profile)
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
}

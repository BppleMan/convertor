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
        let policies = match client {
            ProxyClient::Surge => {
                let mut profile = state
                    .surge_service
                    .try_get_profile(url_builder.clone(), raw_profile)
                    .await?;
                std::mem::take(&mut profile.sorted_policy_list)
            }
            ProxyClient::Clash => {
                let mut profile = state
                    .clash_service
                    .try_get_profile(url_builder.clone(), raw_profile)
                    .await?;
                std::mem::take(&mut profile.sorted_policy_list)
            }
        };
        let raw_url = url_builder.build_raw_url();
        let raw_profile_url = url_builder.build_raw_profile_url()?;
        let profile_url = url_builder.build_profile_url()?;
        let sub_logs_url = url_builder.build_sub_logs_url()?;
        let rule_providers_url = policies
            .iter()
            .map(|policy| url_builder.build_rule_provider_url(policy))
            .collect::<color_eyre::Result<Vec<_>, _>>()?;
        let url_result = UrlResult {
            raw_url,
            raw_profile_url,
            profile_url,
            sub_logs_url,
            rule_providers_url,
        };
        Ok(Json(url_result))
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

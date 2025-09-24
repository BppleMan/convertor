pub mod subscription {
    use crate::server::app_state::AppState;
    use crate::server::response::{ApiError, ApiResponse};
    use crate::server::router::{OptionalScheme, parse_query};
    use axum::extract::{Path, RawQuery, State};
    use axum_extra::extract::Host;
    use axum_extra::headers::HeaderMap;
    use convertor::config::proxy_client::ProxyClient;
    use convertor::config::subscription_config::Headers;
    use convertor::url::url_builder::UrlBuilder;
    use convertor::url::url_result::UrlResult;
    use std::sync::Arc;
    use tracing::instrument;

    #[instrument(skip_all)]
    pub async fn subscription(
        Path(client): Path<ProxyClient>,
        Host(host): Host,
        scheme: Option<OptionalScheme>,
        header_map: HeaderMap,
        State(state): State<Arc<AppState>>,
        RawQuery(query_string): RawQuery,
    ) -> Result<ApiResponse<UrlResult>, ApiError> {
        let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_subscription()?;
        let url_builder = UrlBuilder::from_convertor_query(query, &state.config.secret, client)?;
        let sub_url = url_builder.build_raw_url();
        let headers = Headers::from_header_map(header_map).patch(&state.config.subscription.headers);
        let raw_profile = state.provider.get_raw_profile(sub_url.into(), headers).await?;
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
        let rule_providers_url = policies
            .iter()
            .map(|policy| url_builder.build_rule_provider_url(policy))
            .collect::<color_eyre::Result<Vec<_>, _>>()?;
        let url_result = UrlResult {
            raw_url,
            raw_profile_url,
            profile_url,
            rule_providers_url,
        };
        Ok(ApiResponse::ok(url_result))
    }
}

pub mod subscription {
    use crate::server::app_state::AppState;
    use crate::server::error::AppError;
    use crate::server::router::{OptionalScheme, parse_query};
    use axum::extract::{Path, RawQuery, State};
    use axum::http::{HeaderValue, StatusCode, header};
    use axum::response::{IntoResponse, Response};
    use axum_extra::TypedHeader;
    use axum_extra::extract::Host;
    use axum_extra::headers::UserAgent;
    use convertor::config::client_config::ProxyClient;
    use convertor::config::provider_config::Provider;
    use convertor::provider_api::BosLifeLogs;
    use convertor::url::url_builder::UrlBuilder;
    use convertor::url::url_result::UrlResult;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio_util::bytes::{BufMut, BytesMut};

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct ApiResponse<T>
    where
        T: serde::Serialize,
    {
        pub status: isize,
        pub message: String,
        pub data: Option<T>,
    }

    #[allow(unused)]
    pub async fn subscription(
        Path((client, provider)): Path<(ProxyClient, Provider)>,
        Host(host): Host,
        scheme: Option<OptionalScheme>,
        TypedHeader(user_agent): TypedHeader<UserAgent>,
        State(state): State<Arc<AppState>>,
        RawQuery(query_string): RawQuery,
    ) -> Result<ApiResponse<UrlResult>, AppError> {
        let query = parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_subscription()?;
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
        Ok(ApiResponse::ok(url_result))
    }

    pub async fn sub_logs(
        Path(provider): Path<Provider>,
        Host(host): Host,
        scheme: Option<OptionalScheme>,
        State(state): State<Arc<AppState>>,
        RawQuery(query_string): RawQuery,
    ) -> Result<ApiResponse<BosLifeLogs>, AppError> {
        parse_query(state.as_ref(), scheme, host.as_str(), query_string)?.check_for_sub_logs(&state.config.secret)?;
        let api = state.api_map.get(&provider).ok_or_else(|| AppError::NoSubProvider)?;
        match api.get_sub_logs().await {
            Ok(sub_logs) => Ok(ApiResponse::ok(sub_logs)),
            Err(err) => Ok(ApiResponse::error_with_message(format!("{err:?}"))),
        }
    }

    impl<T> ApiResponse<T>
    where
        T: serde::Serialize,
    {
        pub fn ok(data: T) -> Self {
            Self {
                status: 0,
                message: "ok".to_string(),
                data: Some(data),
            }
        }

        pub fn ok_with_message(data: Option<T>, message: impl AsRef<str>) -> Self {
            Self {
                status: 0,
                message: message.as_ref().to_string(),
                data,
            }
        }
    }

    impl<T> ApiResponse<T>
    where
        T: serde::Serialize,
    {
        pub fn error() -> Self {
            Self {
                status: -1,
                message: "error".to_string(),
                data: None,
            }
        }

        pub fn error_with_message(message: impl AsRef<str>) -> Self {
            Self {
                status: -1,
                message: message.as_ref().to_string(),
                data: None,
            }
        }
    }

    impl<T> IntoResponse for ApiResponse<T>
    where
        T: Serialize,
    {
        fn into_response(self) -> Response {
            // Extracted into separate fn so it's only compiled once for all T.
            fn make_response(buf: BytesMut, ser_result: serde_json::Result<()>) -> Response {
                match ser_result {
                    Ok(()) => (
                        [(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                        )],
                        buf.freeze(),
                    )
                        .into_response(),
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                        )],
                        err.to_string(),
                    )
                        .into_response(),
                }
            }

            // Use a small initial capacity of 128 bytes like serde_json::to_vec
            // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
            let mut buf = BytesMut::with_capacity(128).writer();
            let res = serde_json::to_writer(&mut buf, &self);
            make_response(buf.into_inner(), res)
        }
    }
}

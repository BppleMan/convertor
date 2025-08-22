use crate::server::server_context;
use crate::server::{ExpectPlaceholder, ServerContext, expect_profile};
use axum::body::Body;
use axum::extract::Request;
use color_eyre::eyre::eyre;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::config::sub_provider::SubProvider;
use convertor::core::url_builder::HostPort;
use http_body_util::BodyExt;
use tower::ServiceExt;

pub async fn test_profile(
    server_context: &ServerContext,
    client: ProxyClient,
    provider: SubProvider,
) -> color_eyre::Result<String> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client, provider)?;

    let profile_url = url_builder.build_profile_url();
    let uri = format!("{}?{}", profile_url.path, profile_url.query.encode_to_query_string());
    let request = Request::builder().uri(uri).method("GET").body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    Ok(actual)
}

#[tokio::test]
pub async fn test_surge_boslife() -> color_eyre::Result<()> {
    let server_context = server_context();
    let actual = test_profile(&server_context, ProxyClient::Surge, SubProvider::BosLife).await?;
    insta::with_settings!({ filters => vec![
        (r"(?<=\b[^/\s:\[\]]+):\d{2,5}", ":<port>"),
        (r"(?<=\[[0-9a-fA-F:]+\]):\d{2,5}", ":<port>")
    ]}, {
        insta::assert_snapshot!(actual);
    });
    // insta::assert_snapshot!(actual);
    Ok(())
}

#[tokio::test]
pub async fn test_clash_boslife() -> color_eyre::Result<()> {
    let server_context = server_context();
    test_profile(&server_context, ProxyClient::Clash, SubProvider::BosLife).await?;
    Ok(())
}

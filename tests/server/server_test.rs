use crate::server::server_context;
use crate::server::{ExpectPlaceholder, ServerContext, expect_profile, expect_rule_provider};
use axum::body::Body;
use axum::extract::Request;
use color_eyre::eyre::eyre;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::config::sub_provider::SubProvider;
use convertor::core::profile::policy::Policy;
use convertor::core::url_builder::HostPort;
use http_body_util::BodyExt;
use rstest::rstest;
use tower::ServiceExt;

#[rstest]
#[tokio::test]
pub async fn test_profile(
    server_context: &ServerContext,
    #[values(ProxyClient::Surge, ProxyClient::Clash)] client: ProxyClient,
    #[values(SubProvider::BosLife)] provider: SubProvider,
) -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client, provider)?;

    let client_config = app_state
        .config
        .clients
        .get(&client)
        .ok_or_else(|| eyre!("没有找到对应的订阅提供者: {provider}"))?;
    let convertor_url = url_builder.build_sub_url()?;
    let expect_placeholder = ExpectPlaceholder {
        server: convertor_url.server.to_string(),
        interval: client_config.interval(),
        strict: client_config.strict(),
        uni_sub_host: convertor_url.query.uni_sub_url.host_port()?,
        enc_uni_sub_url: convertor_url.query.encoded_uni_sub_url(),
    };
    let uri = format!(
        "{}?{}",
        convertor_url.path,
        convertor_url.query.encode_to_query_string()
    );

    let request = Request::builder().uri(uri).method("GET").body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_profile(client, &expect_placeholder);

    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[rstest]
#[tokio::test]
pub async fn test_rule_provider(
    server_context: &ServerContext,
    #[values(ProxyClient::Surge, ProxyClient::Clash)] client: ProxyClient,
    #[values(SubProvider::BosLife)] provider: SubProvider,
    #[values(
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::direct_policy(None),
        Policy::direct_policy(Some("no-resolve")),
        Policy::direct_policy(Some("force-remote-dns"))
    )]
    policy: Policy,
) -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client, provider)?;

    let client_config = app_state
        .config
        .clients
        .get(&client)
        .ok_or_else(|| eyre!("没有找到对应的订阅提供者: {provider}"))?;
    let rule_provider_url = url_builder.build_rule_provider_url(&policy)?;
    let expect_placeholder = ExpectPlaceholder {
        server: rule_provider_url.server.to_string(),
        interval: client_config.interval(),
        strict: client_config.strict(),
        uni_sub_host: rule_provider_url.query.uni_sub_url.host_port()?,
        enc_uni_sub_url: rule_provider_url.query.encoded_uni_sub_url(),
    };
    let uri = format!(
        "{}?{}",
        rule_provider_url.path,
        rule_provider_url.query.encode_to_query_string()
    );

    let request = Request::builder().uri(uri).method("GET").body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_rule_provider(client, &policy, &expect_placeholder);

    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

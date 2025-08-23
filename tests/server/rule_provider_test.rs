use crate::init_test;
use crate::server::{ServerContext, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::common::config::provider::SubProvider;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::core::profile::policy::Policy;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::url_builder::HostPort;
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn rule_provider(
    server_context: &ServerContext,
    client: ProxyClient,
    provider: SubProvider,
    policy: Policy,
) -> color_eyre::Result<String> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client, provider)?;

    let rule_provider_url = url_builder.build_rule_provider_url(&policy)?;
    let request = Request::builder()
        .uri(rule_provider_url.to_string())
        .method("GET")
        .header("host", "127.0.0.1")
        .header("user-agent", concat!("convertor/", env!("CARGO_PKG_VERSION")))
        .body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes())
        .to_string()
        .replace(&url_builder.sub_url.host_port()?, "localhost");

    Ok(actual)
}

fn policies() -> [Policy; 7] {
    [
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::direct_policy(None),
        Policy::direct_policy(Some("no-resolve")),
        Policy::direct_policy(Some("force-remote-dns")),
    ]
}

#[tokio::test]
async fn test_rule_provider_surge_boslife() -> color_eyre::Result<()> {
    init_test();
    let server_context = start_server().await?;
    let policies = policies();
    for policy in policies {
        let ctx = format!(
            "test_rule_provider_surge_boslife_{}",
            ClashRenderer::render_provider_name_for_policy(&policy)?
        );
        let actual = rule_provider(&server_context, ProxyClient::Surge, SubProvider::BosLife, policy).await?;
        insta::assert_snapshot!(ctx, actual);
    }
    Ok(())
}

#[tokio::test]
async fn test_rule_provider_clash_boslife() -> color_eyre::Result<()> {
    init_test();
    let server_context = start_server().await?;
    let policies = policies();
    for policy in policies {
        let ctx = format!(
            "test_rule_provider_clash_boslife_{}",
            ClashRenderer::render_provider_name_for_policy(&policy)?
        );
        let actual = rule_provider(&server_context, ProxyClient::Clash, SubProvider::BosLife, policy).await?;
        insta::assert_snapshot!(ctx, actual);
    }
    Ok(())
}

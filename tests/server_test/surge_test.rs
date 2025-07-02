use crate::server_test::ServerContext;
use crate::{start_server, SURGE_MOCK_STR};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::config::surge_config::SurgeConfig;
use convertor::profile::core::policy::Policy;
use convertor::profile::renderer::surge_renderer::SurgeRenderer;
use convertor::profile::surge_profile::SurgeProfile;
use convertor::subscription::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use pretty_assertions::assert_str_eq;
use std::collections::HashMap;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext {
        app,
        app_state,
        base_dir: _base_dir,
    } = start_server(Client::Surge).await?;
    let url_builder = UrlBuilder::new(
        app_state.convertor_config.server.clone(),
        app_state.convertor_config.secret.clone(),
        app_state.subscription_api.get_raw_subscription_url().await?,
    )?;

    let url = url_builder.build_convertor_url(Client::Surge)?;
    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.convertor_config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.clone().oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();

    let mut expect_profile = SurgeProfile::parse(SURGE_MOCK_STR.to_string())?;
    expect_profile.header = SurgeConfig::build_managed_config_header(url_builder.build_convertor_url(Client::Surge)?);
    expect_profile.optimize(url_builder)?;
    let expect = SurgeRenderer::render_profile(&expect_profile)?;

    assert_str_eq!(expect, stream);
    Ok(())
}

#[tokio::test]
pub async fn test_surge_rule_set() -> color_eyre::Result<()> {
    let ServerContext {
        app,
        app_state,
        base_dir: _base_dir,
    } = start_server(Client::Surge).await?;
    let url_builder = UrlBuilder::new(
        app_state.convertor_config.server.clone(),
        app_state.convertor_config.secret.clone(),
        app_state.subscription_api.get_raw_subscription_url().await?,
    )?;
    let policy = Policy {
        name: "BosLife".to_string(),
        option: None,
        is_subscription: false,
    };
    let url = url_builder.build_rule_set_url(Client::Surge, &policy)?;
    println!("{}", url);

    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.convertor_config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();

    println!("{}", stream);
    // assert!(!stream.is_empty());
    Ok(())
}

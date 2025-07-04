use crate::server_test::ServerContext;
use crate::{mock_profile, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::profile::clash_profile::ClashProfile;
use convertor::profile::renderer::clash_renderer::ClashRenderer;
use convertor::subscription::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use std::collections::HashMap;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_clash_profile() -> color_eyre::Result<()> {
    let ServerContext {
        app,
        app_state,
        mock_server,
        ..
    } = start_server(Client::Clash).await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        app_state.api.get_raw_subscription_url().await?,
    )?;

    let url = url_builder.build_convertor_url(Client::Clash)?;
    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).into_owned();

    let raw_profile = mock_profile(Client::Clash, &mock_server)?;
    let mut expect_profile = ClashProfile::template()?;
    expect_profile.optimize(&url_builder, raw_profile, &app_state.config.secret)?;
    let expect = ClashRenderer::render_profile(&expect_profile)?;

    pretty_assertions::assert_str_eq!(expect, stream);
    Ok(())
}

#[tokio::test]
pub async fn test_surge_rule_set() -> color_eyre::Result<()> {
    // let ServerContext { app, app_state } = start_server().await?;
    // let url_builder = UrlBuilder::new(
    //     app_state.convertor_config.server.clone(),
    //     app_state.convertor_config.secret.clone(),
    //     app_state
    //         .subscription_api
    //         .get_raw_subscription_url()
    //         .await?,
    // )?;
    // let url = url_builder
    //     .build_rule_set_url("surge", &RuleSetPolicy::BosLifeSubscription)?;
    // let query_pairs =
    //     serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    // let uri = format!("{}?{}", url.path(), query_pairs);
    // let request = Request::builder()
    //     .uri(&uri)
    //     .header("host", app_state.convertor_config.server_host_with_port()?)
    //     .method("GET")
    //     .body(Body::empty())?;
    // let response = app.oneshot(request).await?;
    // let stream = String::from_utf8_lossy(
    //     &response.into_body().collect().await?.to_bytes(),
    // )
    // .to_string();
    // assert!(!stream.is_empty());
    Ok(())
}

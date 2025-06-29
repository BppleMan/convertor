use crate::server_test::server_context::ServerContext;
use crate::start_server;
use axum::body::Body;
use axum::extract::Request;
use convertor::config::surge_config::SurgeConfig;
use convertor::profile::rule_set_policy::RuleSetPolicy;
use convertor::subscription::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use std::collections::HashMap;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext { app, app_state } = start_server("surge").await?;
    let url_builder = UrlBuilder::new(
        app_state.convertor_config.server.clone(),
        app_state.convertor_config.secret.clone(),
        app_state
            .subscription_api
            .get_raw_subscription_url()
            .await?,
    )?;
    let url = url_builder.build_convertor_url("surge")?;
    let query_pairs =
        serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.convertor_config.server_host_with_port()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(
        &response.into_body().collect().await?.to_bytes(),
    )
    .to_string();
    let expect = SurgeConfig::build_managed_config_header(
        url_builder.build_convertor_url("surge")?,
    );
    assert_eq!(Some(expect.as_str()), stream.lines().next());
    Ok(())
}

#[tokio::test]
pub async fn test_surge_rule_set() -> color_eyre::Result<()> {
    let ServerContext { app, app_state } = start_server("surge").await?;
    let url_builder = UrlBuilder::new(
        app_state.convertor_config.server.clone(),
        app_state.convertor_config.secret.clone(),
        app_state
            .subscription_api
            .get_raw_subscription_url()
            .await?,
    )?;
    let url = url_builder
        .build_rule_set_url("surge", &RuleSetPolicy::BosLifeSubscription)?;
    let query_pairs =
        serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.convertor_config.server_host_with_port()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(
        &response.into_body().collect().await?.to_bytes(),
    )
    .to_string();
    assert!(!stream.is_empty());
    Ok(())
}

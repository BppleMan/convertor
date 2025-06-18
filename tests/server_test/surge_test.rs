use crate::server_test::server_context::ServerContext;
use crate::start_server;
use axum::body::Body;
use axum::extract::Request;
use convertor::config::surge_config::{RuleSetType, SurgeConfig};
use http_body_util::BodyExt;
use std::collections::HashMap;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext { app, service } = start_server().await?;
    let convertor_url = service.get_subscription_url(None).await?;
    let url = convertor_url.build_convertor_url("surge")?;
    let query_pairs =
        serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", service.config.server_host_with_port()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(
        &response.into_body().collect().await?.to_bytes(),
    )
    .to_string();
    let expect = SurgeConfig::build_managed_config_header(
        convertor_url.build_convertor_url("surge")?,
    );
    assert_eq!(Some(expect.as_str()), stream.lines().next());
    Ok(())
}

#[tokio::test]
pub async fn test_surge_rule_set() -> color_eyre::Result<()> {
    let ServerContext { app, service } = start_server().await?;
    let convertor_url = service.get_subscription_url(None).await?;
    let url =
        convertor_url.build_rule_set_url(&RuleSetType::BosLifeSubscription)?;
    let query_pairs =
        serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", service.config.server_host_with_port()?)
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

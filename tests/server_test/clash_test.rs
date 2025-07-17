use crate::server_test::ServerContext;
use crate::{expect_profile, expect_rule_provider, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::core::profile::policy::Policy;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_clash_profile() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let url_builder = app_state.config.create_url_builder()?;

    let url = url_builder.build_convertor_url(Client::Clash)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_profile(Client::Clash, url_builder.encode_encrypted_raw_sub_url());
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[tokio::test]
pub async fn test_clash_boslife_policy_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let url_builder = app_state.config.create_url_builder()?;
    let policy = Policy {
        name: "BosLife".to_string(),
        option: None,
        is_subscription: false,
    };

    let url = url_builder.build_rule_provider_url(Client::Clash, &policy)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_rule_provider(Client::Clash, &policy);
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[tokio::test]
pub async fn test_clash_subscription_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let url_builder = app_state.config.create_url_builder()?;
    let policy = Policy::subscription_policy();

    let url = url_builder.build_rule_provider_url(Client::Clash, &policy)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    pretty_assertions::assert_str_eq!(
        format!("payload:\n    - DOMAIN,http://{}", url_builder.sub_host()?),
        actual.trim()
    );

    Ok(())
}

#[tokio::test]
pub async fn test_clash_direct_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let url_builder = app_state.config.create_url_builder()?;
    let policy = Policy::direct_policy();
    let url = url_builder.build_rule_provider_url(Client::Clash, &policy)?;

    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_rule_provider(Client::Clash, &policy);
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

use crate::server_test::ServerContext;
use crate::{expect_profile, expect_rule_provider, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::core::profile::policy::Policy;
use convertor::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Surge)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;

    let url = url_builder.build_convertor_url(Client::Surge)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_profile(Client::Surge, &url_builder.encode_encrypted_raw_sub_url());
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[tokio::test]
pub async fn test_surge_boslife_policy_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Surge)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;
    let policy = Policy {
        name: "BosLife".to_string(),
        option: None,
        is_subscription: false,
    };

    let url = url_builder.build_rule_provider_url(Client::Surge, &policy)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_rule_provider(Client::Surge, &policy);
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[tokio::test]
pub async fn test_surge_subscription_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Surge)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;
    let policy = Policy::subscription_policy();

    let url = url_builder.build_rule_provider_url(Client::Surge, &policy)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    pretty_assertions::assert_str_eq!(format!("DOMAIN,http://{}", url_builder.sub_host()?), actual.trim());

    Ok(())
}

#[tokio::test]
pub async fn test_surge_direct_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Surge)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;
    let policy = Policy::direct_policy();

    let url = url_builder.build_rule_provider_url(Client::Surge, &policy)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_rule_provider(Client::Surge, &policy);
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

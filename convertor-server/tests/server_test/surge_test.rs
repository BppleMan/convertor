use crate::{ServerContext, expect_profile, expect_rule_provider, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor_core::client::Client;
use convertor_core::core::profile::policy::Policy;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let convertor_url = app_state.config.create_convertor_url(Client::Surge)?;

    let url = convertor_url.build_sub_url()?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let expect = expect_profile(Client::Surge, convertor_url.encoded_raw_sub_url()?);
    pretty_assertions::assert_str_eq!(expect, actual);

    Ok(())
}

#[tokio::test]
pub async fn test_surge_boslife_policy_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let convertor_url = app_state.config.create_convertor_url(Client::Surge)?;
    let policy = Policy {
        name: "BosLife".to_string(),
        option: None,
        is_subscription: false,
    };

    let url = convertor_url.build_rule_provider_url(&policy)?;
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
    let convertor_url = app_state.config.create_convertor_url(Client::Surge)?;
    let policy = Policy::subscription_policy();

    let url = convertor_url.build_rule_provider_url(&policy)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    pretty_assertions::assert_str_eq!(
        format!("DOMAIN,http://{}", convertor_url.raw_sub_host()?),
        actual.trim()
    );

    Ok(())
}

#[tokio::test]
pub async fn test_surge_direct_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let convertor_url = app_state.config.create_convertor_url(Client::Surge)?;
    let policy = Policy::direct_policy();

    let url = convertor_url.build_rule_provider_url(&policy)?;
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

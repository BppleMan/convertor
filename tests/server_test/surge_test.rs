use crate::server_test::ServerContext;
use crate::{mock_profile, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::config::surge_config::SurgeConfig;
use convertor::profile::core::policy::Policy;
use convertor::profile::core::rule::RuleType;
use convertor::profile::parser::surge_parser::SurgeParser;
use convertor::profile::renderer::surge_renderer::SurgeRenderer;
use convertor::profile::surge_profile::SurgeProfile;
use convertor::subscription::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use std::collections::HashMap;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext {
        app,
        app_state,
        mock_server,
        ..
    } = start_server(Client::Surge).await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        app_state.api.get_raw_subscription_url().await?,
    )?;

    let url = url_builder.build_convertor_url(Client::Surge)?;
    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();

    let mut expect_profile = SurgeProfile::parse(mock_profile(Client::Surge, &mock_server)?)?;
    expect_profile.header = SurgeConfig::build_managed_config_header(url_builder.build_convertor_url(Client::Surge)?);
    expect_profile.optimize(url_builder)?;
    let expect = SurgeRenderer::render_profile(&expect_profile)?;

    pretty_assertions::assert_str_eq!(expect, stream);
    Ok(())
}

#[tokio::test]
pub async fn test_surge_rule_set() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        app_state.api.get_raw_subscription_url().await?,
    )?;
    let policy = Policy {
        name: "BosLife".to_string(),
        option: None,
        is_subscription: false,
    };
    let url = url_builder.build_rule_set_url(Client::Surge, &policy)?;

    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_str_eq!("[Rule]", lines[0]);
    pretty_assertions::assert_eq!(896, lines.len());

    let rules = SurgeParser::parse_rules(lines)?;
    for rule in rules {
        pretty_assertions::assert_eq!(&policy, &rule.policy);
    }
    Ok(())
}

#[tokio::test]
pub async fn test_surge_subscription_rule_set() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        app_state.api.get_raw_subscription_url().await?,
    )?;
    let policy = Policy::subscription_policy();
    let url = url_builder.build_rule_set_url(Client::Surge, &policy)?;

    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    println!("{}", stream);
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_str_eq!("[Rule]", lines[0]);
    pretty_assertions::assert_eq!(3, lines.len());

    let rules = SurgeParser::parse_rules(lines)?;
    for rule in rules {
        pretty_assertions::assert_eq!(&RuleType::Domain, &rule.rule_type);
        pretty_assertions::assert_eq!(&policy.name, &rule.policy.name);
        pretty_assertions::assert_eq!(&policy.option, &rule.policy.option);
    }
    Ok(())
}

#[tokio::test]
pub async fn test_surge_direct_rule_set() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Surge).await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        app_state.api.get_raw_subscription_url().await?,
    )?;
    let policy = Policy::direct_policy();
    let url = url_builder.build_rule_set_url(Client::Surge, &policy)?;

    let query_pairs = serde_qs::to_string(&url.query_pairs().collect::<HashMap<_, _>>())?;
    let uri = format!("{}?{}", url.path(), query_pairs);
    let request = Request::builder()
        .uri(&uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_str_eq!("[Rule]", lines[0]);
    pretty_assertions::assert_eq!(7, lines.len());

    let rules = SurgeParser::parse_rules(lines)?;
    pretty_assertions::assert_eq!(2, rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&RuleType::Domain, &rule.rule_type);
        pretty_assertions::assert_eq!(&policy.name, &rule.policy.name);
        pretty_assertions::assert_eq!(&policy.option, &rule.policy.option);
    }
    Ok(())
}

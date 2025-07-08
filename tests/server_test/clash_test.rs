use crate::server_test::ServerContext;
use crate::{count_rule_lines, mock_profile, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::profile::core::clash_profile::ClashProfile;
use convertor::profile::core::policy::Policy;
use convertor::profile::core::profile::Profile;
use convertor::profile::core::rule::RuleType;
use convertor::profile::parser::clash_parser::ClashParser;
use convertor::profile::renderer::Renderer;
use convertor::profile::renderer::clash_renderer::ClashRenderer;
use convertor::subscription::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_clash_profile() -> color_eyre::Result<()> {
    let ServerContext {
        app,
        app_state,
        mock_server,
        ..
    } = start_server(Client::Clash).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Clash)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;

    let url = url_builder.build_convertor_url(Client::Clash)?;
    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).into_owned();

    let raw_profile = mock_profile(Client::Clash, &mock_server)?;
    let mut expect_profile = ClashProfile::template()?;
    expect_profile.optimize(&url_builder, Some(raw_profile), Some(&app_state.config.secret))?;
    let expect = ClashRenderer::render_profile(&expect_profile)?;

    pretty_assertions::assert_str_eq!(expect, stream);
    Ok(())
}

#[tokio::test]
pub async fn test_clash_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Clash)
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
    let url = url_builder.build_rule_provider_url(Client::Clash, &policy)?;

    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_str_eq!("payload:", lines[0]);
    pretty_assertions::assert_eq!(972, lines.len());

    let rules = ClashParser::parse_rules(&stream)?;
    pretty_assertions::assert_eq!(count_rule_lines(Client::Clash, &policy), rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&policy, &rule.policy);
    }
    Ok(())
}

#[tokio::test]
pub async fn test_clash_subscription_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Clash)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;
    let policy = Policy::subscription_policy();
    let url = url_builder.build_rule_provider_url(Client::Clash, &policy)?;

    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_str_eq!("payload:", lines[0]);
    pretty_assertions::assert_eq!(3, lines.len());

    let rules = ClashParser::parse_rules(stream)?;
    pretty_assertions::assert_eq!(1, rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&RuleType::Domain, &rule.rule_type);
        pretty_assertions::assert_eq!(&policy.name, &rule.policy.name);
        pretty_assertions::assert_eq!(&policy.option, &rule.policy.option);
    }
    Ok(())
}

#[tokio::test]
pub async fn test_clash_direct_rule_provider() -> color_eyre::Result<()> {
    let ServerContext { app, app_state, .. } = start_server(Client::Clash).await?;
    let service_config = &app_state.config.service_config;
    let raw_sub_url = app_state
        .api
        .get_raw_sub_url(service_config.base_url.clone(), Client::Clash)
        .await?;
    let url_builder = UrlBuilder::new(
        app_state.config.server.clone(),
        app_state.config.secret.clone(),
        raw_sub_url,
    )?;
    let policy = Policy::direct_policy();
    let url = url_builder.build_rule_provider_url(Client::Clash, &policy)?;

    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_str_eq!("payload:", lines[0]);
    pretty_assertions::assert_eq!(8, lines.len());

    let rules = ClashParser::parse_rules(stream)?;
    pretty_assertions::assert_eq!(6, rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&policy.name, &rule.policy.name);
        pretty_assertions::assert_eq!(&policy.option, &rule.policy.option);
    }
    Ok(())
}

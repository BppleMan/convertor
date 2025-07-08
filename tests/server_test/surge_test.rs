use crate::server_test::ServerContext;
use crate::{count_rule_lines, expect_profile, mock_profile, start_server};
use axum::body::Body;
use axum::extract::Request;
use convertor::client::Client;
use convertor::config::surge_config::SurgeConfig;
use convertor::encrypt::encrypt;
use convertor::profile::core::policy::Policy;
use convertor::profile::core::profile::Profile;
use convertor::profile::core::rule::RuleType;
use convertor::profile::core::surge_profile::SurgeProfile;
use convertor::profile::parser::surge_parser::SurgeParser;
use convertor::profile::renderer::Renderer;
use convertor::profile::renderer::surge_renderer::SurgeRenderer;
use convertor::subscription::url_builder::UrlBuilder;
use http_body_util::BodyExt;
use percent_encoding::utf8_percent_encode;
use tower::ServiceExt;

#[tokio::test]
pub async fn test_surge_profile() -> color_eyre::Result<()> {
    let ServerContext {
        app,
        app_state,
        mock_server,
        ..
    } = start_server(Client::Surge).await?;
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
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();

    let expect = expect_profile(
        Client::Surge,
        // &url_builder.encrypted_raw_sub_url,
        utf8_percent_encode(
            &utf8_percent_encode(&url_builder.encrypted_raw_sub_url, percent_encoding::NON_ALPHANUMERIC).to_string(),
            percent_encoding::CONTROLS,
        )
        .to_string(),
    );

    pretty_assertions::assert_str_eq!(expect, stream);
    Ok(())
}

#[tokio::test]
pub async fn test_surge_rule_provider() -> color_eyre::Result<()> {
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
    println!("url: {}", url);

    let uri = format!("{}?{}", url.path(), url.query().expect("必须有查询参数"));
    let request = Request::builder()
        .uri(uri)
        .header("host", app_state.config.server_addr()?)
        .method("GET")
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_eq!(854, lines.len());

    let rules = SurgeParser::parse_rules_for_provider(lines)?;
    pretty_assertions::assert_eq!(count_rule_lines(Client::Surge, &policy), rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&policy, &rule.policy);
    }
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
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_eq!(2, lines.len());

    let rules = SurgeParser::parse_rules_for_provider(lines)?;
    pretty_assertions::assert_eq!(1, rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&RuleType::Domain, &rule.rule_type);
        pretty_assertions::assert_eq!(&policy.name, &rule.policy.name);
        pretty_assertions::assert_eq!(&policy.option, &rule.policy.option);
    }
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
    let stream = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let lines = stream.lines().collect::<Vec<_>>();

    pretty_assertions::assert_eq!(2, lines.len());

    let rules = SurgeParser::parse_rules_for_provider(lines)?;
    pretty_assertions::assert_eq!(1, rules.len());
    for rule in rules {
        pretty_assertions::assert_eq!(&RuleType::Domain, &rule.rule_type);
        pretty_assertions::assert_eq!(true, rule.value.is_some());
    }
    Ok(())
}

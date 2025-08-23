use crate::init_test;
use convertor::common::config::provider::SubProvider;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::core::profile::policy::Policy;
use convertor::core::url_builder::UrlBuilder;
use url::Url;

fn test_url_builder(client: ProxyClient, provider: SubProvider) -> color_eyre::Result<()> {
    let server = Url::parse("http://127.0.0.1:8001")?;
    let sub_url = Url::parse("https://example.com/subscription?token=12345")?;
    let secret = "my_secret_key";
    let url_builder = UrlBuilder::new(
        secret,
        None,
        client,
        provider,
        server.clone(),
        sub_url.clone(),
        None,
        86400,
        true,
    )?;
    let encoded_sub_url = url_builder.enc_sub_url.clone();

    let raw_sub_url = url_builder.build_raw_url();
    pretty_assertions::assert_str_eq!(format!("{sub_url}&flag={client}"), raw_sub_url.to_string());

    let sub_url = url_builder.build_profile_url()?;
    pretty_assertions::assert_eq!(
        format!(
            "{server}profile?client={client}&provider={provider}&server={server}&interval=86400&strict=true&sub_url={encoded_sub_url}",
        ),
        sub_url.to_string()
    );

    let rule_provider_url = url_builder.build_rule_provider_url(&Policy::subscription_policy())?;
    pretty_assertions::assert_eq!(
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval=86400&policy.name=DIRECT&policy.is_subscription=true&sub_url={encoded_sub_url}",
        ),
        rule_provider_url.to_string()
    );

    Ok(())
}

#[test]
fn test_url_builder_surge_boslife() -> color_eyre::Result<()> {
    init_test();
    test_url_builder(ProxyClient::Surge, SubProvider::BosLife)
}

use crate::init_test;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::config::sub_provider::SubProvider;
use convertor::core::profile::policy::Policy;
use convertor::core::url_builder::UrlBuilder;
use rstest::rstest;
use url::Url;

#[rstest]
fn test_url_builder(
    #[values(ProxyClient::Surge, ProxyClient::Clash)] client: ProxyClient,
    #[values(SubProvider::BosLife)] provider: SubProvider,
) -> color_eyre::Result<()> {
    init_test();

    let server = Url::parse("http://127.0.0.1:8001")?;
    let uni_sub_url = Url::parse("https://example.com/subscription?token=12345")?;
    let secret = "my_secret_key";
    let url_builder = UrlBuilder::new(
        secret,
        client,
        provider,
        server.clone(),
        uni_sub_url.clone(),
        None,
        86400,
        true,
    )?;

    let raw_sub_url = url_builder.build_raw_url();
    pretty_assertions::assert_str_eq!(format!("{uni_sub_url}&flag={client}"), raw_sub_url.to_string());

    let sub_url = url_builder.build_profile_url();
    let encoded_uni_sub_url = sub_url.query.encoded_uni_sub_url();
    pretty_assertions::assert_eq!(
        format!(
            "{server}profile?client={client}&provider={provider}&server={server}&interval=86400&strict=true&uni_sub_url={encoded_uni_sub_url}"
        ),
        sub_url.to_string()
    );

    let rule_provider_url = url_builder.build_rule_provider_url(&Policy::subscription_policy());
    pretty_assertions::assert_eq!(
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval=86400&policy.name=DIRECT&policy.is_subscription=true&uni_sub_url={encoded_uni_sub_url}",
        ),
        rule_provider_url.to_string()
    );

    Ok(())
}

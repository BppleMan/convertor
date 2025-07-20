use crate::init_test;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::url::ConvertorUrl;
use convertor::core::profile::policy::Policy;
use url::Url;

#[test]
fn test_url_builder() -> color_eyre::Result<()> {
    init_test();

    let server = Url::parse("http://127.0.0.1:8001")?;
    let raw_sub_url = Url::parse("https://example.com/subscription?token=12345")?;
    let secret = "my_secret_key";
    let convertor_url = ConvertorUrl::new(
        secret,
        ProxyClient::Surge,
        server.clone(),
        raw_sub_url.clone(),
        86400,
        true,
        None,
    )?;

    let raw_sub_url = convertor_url.build_raw_sub_url()?;
    pretty_assertions::assert_str_eq!(
        "https://example.com/subscription?token=12345&flag=surge",
        raw_sub_url.as_str()
    );

    let sub_url = convertor_url.build_sub_url()?;
    let encoded_uni_sub_url = convertor_url.encoded_uni_sub_url()?;
    pretty_assertions::assert_eq!(
        format!(
            "http://127.0.0.1:8001/profile?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&uni_sub_url={}",
            encoded_uni_sub_url
        ),
        sub_url.to_string()
    );

    let rule_provider_url = convertor_url.build_rule_provider_url(&Policy::subscription_policy())?;
    pretty_assertions::assert_eq!(
        format!(
            "http://127.0.0.1:8001/rule-provider?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&policy.name=DIRECT&policy.is_subscription=true&uni_sub_url={}",
            encoded_uni_sub_url
        ),
        rule_provider_url.to_string()
    );

    Ok(())
}

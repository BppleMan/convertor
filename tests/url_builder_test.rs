use convertor::client::Client;
use convertor::core::profile::policy::Policy;
use convertor::url_builder::UrlBuilder;
use tracing::warn;
use url::Url;

#[test]
pub fn test_url_builder() -> color_eyre::Result<()> {
    if let Err(e) = color_eyre::install() {
        warn!("Failed to install color_eyre: {}", e);
    };

    let server = Url::parse("http://127.0.0.1:8001")?;
    let raw_sub_url = Url::parse("https://example.com/subscription?token=12345")?;
    let secret = "my_secret_key";
    let url_builder = UrlBuilder::new(server.clone(), secret, raw_sub_url.clone(), 86400, true)?;
    let encoded_raw_sub_url = url_builder.encode_encrypted_raw_sub_url();

    let raw_sub_url = url_builder.build_raw_sub_url(Client::Surge)?;
    pretty_assertions::assert_str_eq!(
        "https://example.com/subscription?token=12345&flag=surge",
        raw_sub_url.as_str()
    );

    let convertor_url = url_builder.build_convertor_url(Client::Surge)?;
    pretty_assertions::assert_eq!(
        format!(
            "http://127.0.0.1:8001/profile?client=surge&original_host=http://127.0.0.1:8001/&interval=86400&strict=true&raw_sub_url={}",
            encoded_raw_sub_url
        ),
        convertor_url.to_string()
    );

    let rule_provider_url = url_builder.build_rule_provider_url(Client::Surge, &Policy::subscription_policy())?;
    pretty_assertions::assert_eq!(
        format!(
            "http://127.0.0.1:8001/rule-provider?client=surge&original_host=http://127.0.0.1:8001/&interval=86400&strict=true&policy.name=DIRECT&policy.is_subscription=true&raw_sub_url={}",
            encoded_raw_sub_url
        ),
        rule_provider_url.to_string()
    );

    Ok(())
}

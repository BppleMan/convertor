use color_eyre::eyre::eyre;
use convertor::client::Client;
use convertor::encrypt::decrypt;
use convertor::profile::core::policy::Policy;
use convertor::router::query::ProfileQuery;
use convertor::subscription::url_builder::UrlBuilder;
use tracing::warn;
use url::Url;

fn create_url_builder() -> color_eyre::Result<(String, UrlBuilder)> {
    if let Err(e) = color_eyre::install() {
        warn!("Failed to install color_eyre: {}", e);
    };

    let server = Url::parse("http://127.0.0.1:8001")?;
    let raw_sub_url = Url::parse("https://example.com/subscription?token=12345")?;
    let secret = "my_secret_key";
    Ok((
        secret.to_string(),
        UrlBuilder::new(server.clone(), secret, raw_sub_url.clone())?,
    ))
}

#[test]
pub fn test_build_raw_sub_url() -> color_eyre::Result<()> {
    let (_, url_builder) = create_url_builder()?;

    let subscription_url = url_builder.build_subscription_url(Client::Surge)?;
    pretty_assertions::assert_str_eq!(
        format!("{}&flag=surge", url_builder.raw_sub_url),
        subscription_url.as_str()
    );

    Ok(())
}

#[test]
pub fn test_build_convertor_url() -> color_eyre::Result<()> {
    let (secret, url_builder) = create_url_builder()?;

    let convertor_url = url_builder.build_convertor_url(Client::Surge)?;
    pretty_assertions::assert_str_eq!("http", convertor_url.scheme());
    pretty_assertions::assert_str_eq!("127.0.0.1", convertor_url.host_str().expect("必须有 host"));
    pretty_assertions::assert_str_eq!("8001", convertor_url.port().expect("必须有 port").to_string());
    let profile_query = ProfileQuery::decode_from_query_string(
        convertor_url
            .query()
            .ok_or_else(|| eyre!("convertor url 必须有 query 参数"))?,
    )?;
    pretty_assertions::assert_str_eq!(Client::Surge.to_string(), profile_query.client.to_string());
    pretty_assertions::assert_str_eq!(url_builder.server.as_str(), profile_query.original_host.as_str());
    pretty_assertions::assert_str_eq!(
        url_builder.raw_sub_url.as_str(),
        decrypt(secret.as_bytes(), &profile_query.raw_sub_url)?
    );
    pretty_assertions::assert_eq!(None, profile_query.policy);

    Ok(())
}

#[test]
pub fn test_build_rule_provider_url() -> color_eyre::Result<()> {
    let (secret, url_builder) = create_url_builder()?;

    let rule_provider_url = url_builder.build_rule_provider_url(Client::Surge, &Policy::subscription_policy())?;
    let profile_query = ProfileQuery::decode_from_query_string(
        rule_provider_url
            .query()
            .ok_or_else(|| eyre!("convertor url 必须有 query 参数"))?,
    )?;
    pretty_assertions::assert_str_eq!(Client::Surge.to_string(), profile_query.client.to_string());
    pretty_assertions::assert_str_eq!(url_builder.server.as_str(), profile_query.original_host.as_str());
    pretty_assertions::assert_str_eq!(
        url_builder.raw_sub_url.as_str(),
        decrypt(secret.as_bytes(), &profile_query.raw_sub_url)?
    );
    pretty_assertions::assert_str_eq!("DIRECT", &profile_query.policy.as_ref().unwrap().name);
    pretty_assertions::assert_str_eq!(
        "true",
        profile_query
            .policy
            .as_ref()
            .unwrap()
            .is_subscription
            .to_string()
            .as_str()
    );

    Ok(())
}

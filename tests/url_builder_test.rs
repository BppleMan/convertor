use convertor::client::Client;
use convertor::encrypt::decrypt;
use convertor::profile::core::policy::Policy;
use convertor::server::router::ProfileQuery;
use convertor::subscription::url_builder::UrlBuilder;
use std::collections::HashMap;
use tracing::warn;
use url::Url;

#[test]
pub fn test_url_builder() -> color_eyre::Result<()> {
    if let Err(e) = color_eyre::install() {
        warn!("Failed to install color_eyre: {}", e);
    };

    let server = Url::parse("http://127.0.0.1:8001")?;
    let service_url = Url::parse("https://example.com/subscription?token=12345")?;
    let secret = "my_secret_key";
    let url_builder = UrlBuilder::new(server.clone(), secret, service_url.clone())?;

    let convertor_url = url_builder.build_convertor_url(Client::Surge)?;
    pretty_assertions::assert_str_eq!("http", convertor_url.scheme());
    pretty_assertions::assert_str_eq!("127.0.0.1", convertor_url.host_str().expect("必须有 host"));
    pretty_assertions::assert_str_eq!("8001", convertor_url.port().expect("必须有 port").to_string());
    let query_pairs = convertor_url.query_pairs().collect::<HashMap<_, _>>();
    pretty_assertions::assert_str_eq!(
        service_url.as_str(),
        decrypt(secret.as_bytes(), &query_pairs["raw_url"])?
    );

    let subscription_url = url_builder.build_subscription_url(Client::Surge)?;
    pretty_assertions::assert_str_eq!(
        format!("{}&flag=surge", service_url.as_str()),
        subscription_url.as_str()
    );

    let rule_set_url = url_builder.build_rule_set_url(Client::Surge, &Policy::subscription_policy())?;
    let query: ProfileQuery = serde_qs::from_str(rule_set_url.query().as_ref().unwrap())?;
    pretty_assertions::assert_str_eq!("DIRECT", &query.policy.as_ref().unwrap().name);
    pretty_assertions::assert_str_eq!(
        "true",
        query.policy.as_ref().unwrap().is_subscription.to_string().as_str()
    );
    Ok(())
}

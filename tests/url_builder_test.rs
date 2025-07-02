use convertor::client::Client;
use convertor::encrypt::decrypt;
use convertor::subscription::url_builder::UrlBuilder;
use pretty_assertions::assert_str_eq;
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
    let convertor_url = UrlBuilder::new(server.clone(), secret, service_url.clone())?;

    let con_url = convertor_url.build_convertor_url(Client::Surge)?;
    assert!(con_url
        .as_str()
        .starts_with(&format!("{}surge?raw_url=", server.as_str())));
    let query_pairs = con_url.query_pairs().collect::<HashMap<_, _>>();
    assert_str_eq!(
        "https://example.com/subscription?token=12345",
        decrypt(secret.as_bytes(), &query_pairs["raw_url"])?
    );

    let sub_url = convertor_url.build_subscription_url(Client::Surge)?;
    assert_str_eq!(format!("{}&flag=surge", service_url.as_str()), sub_url.as_str());

    // forin RuleSetPolicy::all() {
    //     let rule_set_url = convertor_url.build_rule_set_url("surge", rst)?;
    //     let start_with = format!("{}surge/rule-set?raw_url", server.as_str());
    //     let end_with = if matches!(rst, RuleSetPolicy::BosLifeSubscription) {
    //         "boslife=true".to_string()
    //     } else {
    //         format!("policies={}", rst.policy().replace(",", "%2C"))
    //     };
    //     assert!(rule_set_url.as_str().starts_with(&start_with));
    //     assert!(rule_set_url.as_str().ends_with(&end_with));
    // }
    Ok(())
}

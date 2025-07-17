use convertor::client::Client;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::core::profile::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::init_backtrace;
use convertor::service_provider::subscription_api::boslife_api::BosLifeApi;
use std::path::Path;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let base_dir = std::env::current_dir()?.join(".convertor.bench");
    init_backtrace();
    // 下面两种方案任选一
    #[cfg(feature = "bench")]
    tracing_span_tree::span_tree().aggregate(true).enable();
    // #[cfg(feature = "bench")]
    // tracing_profile::init_tracing()?;

    let convertor_config = ConvertorConfig::search(&base_dir, Option::<&Path>::None)?;
    let api = BosLifeApi::new(
        &base_dir,
        reqwest::Client::new(),
        convertor_config.service_config.clone(),
    );
    let raw_sub_url = api
        .get_raw_sub_url(convertor_config.service_config.base_url.clone(), Client::Surge)
        .await?;
    let url_builder = convertor_config.create_url_builder(raw_sub_url)?;

    let file = std::fs::read_to_string(base_dir.join("mock.conf"))?;
    let mut profile = SurgeProfile::parse(file)?;
    profile.optimize(&url_builder)?;

    Ok(())
}

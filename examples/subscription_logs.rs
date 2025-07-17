use convertor::config::convertor_config::ConvertorConfig;
use convertor::service_provider::subscription_api::boslife_api::BosLifeApi;
use convertor::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let client = reqwest::Client::new();
    let api = BosLifeApi::new(&base_dir, client, config.service_config.clone());

    let logs = api.get_sub_logs(config.service_config.base_url).await?;
    println!("{logs:#?}");

    Ok(())
}

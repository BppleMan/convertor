use convertor::convertor_config::ConvertorConfig;
use convertor::service_provider::api::ServiceApi;
use convertor::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let client = reqwest::Client::new();
    let api = ServiceApi::get_service_provider_api(config.service_config, &base_dir, client);

    let logs = api.get_sub_logs().await?;
    println!("{logs:#?}");

    Ok(())
}

use convertor::common::once::{init_backtrace, init_base_dir};
use convertor::config::ConvertorConfig;
use convertor::provider_api::ProviderApi;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let api_map = ProviderApi::create_api_no_redis(config.providers);
    let api = api_map
        .get(&convertor::config::provider_config::Provider::BosLife)
        .ok_or_else(|| color_eyre::eyre::eyre!("未找到 BosLife 订阅提供者"))?;

    let logs = api.get_sub_logs().await?;
    println!("{logs:#?}");

    Ok(())
}

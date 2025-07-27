use convertor::api::SubProviderWrapper;
use convertor::common::config::ConvertorConfig;
use convertor::common::once::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let api_map = SubProviderWrapper::create_api(config.providers, &base_dir);
    let api = api_map
        .get(&convertor::common::config::sub_provider::SubProvider::BosLife)
        .ok_or_else(|| color_eyre::eyre::eyre!("未找到 BosLife 订阅提供者"))?;

    let logs = api.get_sub_logs().await?;
    println!("{logs:#?}");

    Ok(())
}

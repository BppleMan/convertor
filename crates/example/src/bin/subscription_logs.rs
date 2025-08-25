use convertor::common::config::ConvertorConfig;
use convertor::common::once::{init_backtrace, init_base_dir};
use convertor::common::redis::{redis_client, redis_url};
use convertor::provider_api::ProviderApi;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let redis = redis_client(redis_url())?;
    let connection_manager = redis::aio::ConnectionManager::new(redis).await?;

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let api_map = ProviderApi::create_api(config.providers, connection_manager);
    let api = api_map
        .get(&convertor::common::config::provider_config::Provider::BosLife)
        .ok_or_else(|| color_eyre::eyre::eyre!("未找到 BosLife 订阅提供者"))?;

    let logs = api.get_sub_logs().await?;
    println!("{logs:#?}");

    Ok(())
}

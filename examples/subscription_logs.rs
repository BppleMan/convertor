use convertor::api::SubProviderApi;
use convertor::common::config::ConvertorConfig;
use convertor::common::once::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let api = SubProviderApi::get_service_provider_api(config.provider, &base_dir);

    let logs = api.get_sub_logs().await?;
    println!("{logs:#?}");

    Ok(())
}

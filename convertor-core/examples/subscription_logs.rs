use convertor_core::api::ServiceApi;
use convertor_core::config::ConvertorConfig;
use convertor_core::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let api = ServiceApi::get_service_provider_api(config.service_config, &base_dir);

    let logs = api.get_sub_logs().await?;
    println!("{logs:#?}");

    Ok(())
}

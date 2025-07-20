use crate::init_test;
use crate::server::mock::start_mock_provider_server;
use color_eyre::Report;
use convertor::api::SubProviderApi;
use convertor::cli::sub_provider_executor::{SubProviderCmd, SubProviderExecutor};
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client::ProxyClient;
use std::path::Path;

trait TestConfig {
    fn test(base_dir: impl AsRef<Path>) -> Result<ConvertorConfig, Report>;
}

impl TestConfig for ConvertorConfig {
    fn test(base_dir: impl AsRef<Path>) -> Result<ConvertorConfig, Report> {
        let base_dir = base_dir.as_ref();
        let mut convertor_config = ConvertorConfig::template()?;
        if let Some(surge_config) = convertor_config.client.surge.as_mut() {
            surge_config.set_surge_dir(base_dir.join("provider/surge").display().to_string());
        };
        if let Some(clash_config) = convertor_config.client.clash.as_mut() {
            clash_config.set_clash_dir(base_dir.join("provider/clash").display().to_string());
        };
        Ok(convertor_config)
    }
}

/// convertor sub surge
#[tokio::test]
async fn test_sub_surge() -> Result<(), Report> {
    let base_dir = init_test();
    let convertor_config = ConvertorConfig::test(&base_dir)?;
    let api = SubProviderApi::get_service_provider_api(convertor_config.provider.clone(), &base_dir);
    let mut executor = SubProviderExecutor::new(convertor_config, api);

    let mock_server = start_mock_provider_server(&executor.config.provider).await?;
    executor
        .config
        .provider
        .uni_sub_url
        .set_port(Some(mock_server.port()))
        .map_err(|_| Report::msg("can't set mock server port"))?;
    executor.config.provider.api_host = mock_server.base_url().parse()?;
    let cmd = SubProviderCmd {
        client: ProxyClient::Surge,
        ..Default::default()
    };
    executor.execute(cmd).await?;
    Ok(())
}

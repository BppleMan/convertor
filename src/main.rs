use clap::Parser;
use color_eyre::Result;
use convertor::api::ServiceApi;
use convertor::common::config::ConvertorConfig;
use convertor::server::{ConvertorServer, start_server};
use convertor::{init_backtrace, init_base_dir, init_log};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(&base_dir);

    let cli = ConvertorServer::parse();
    let config = ConvertorConfig::search(&base_dir, cli.config)?;
    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir);

    start_server(cli.listen, config, api, &base_dir).await?;

    Ok(())
}

// #[tokio::main(flavor = "multi_thread")]
// async fn main() -> Result<(), Report> {
//     let base_dir = init_base_dir();
//     init_backtrace();
//
//     let cli = ConvertorCli::parse();
//     let config = ConvertorConfig::search(&base_dir, None::<&str>)?;
//     let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir);
//
//     match cli {
//         ConvertorCli::Subscription(args) => {
//             let mut subscription_service = SubscriptionService { config, api };
//             subscription_service.execute(args).await?;
//         }
//         ConvertorCli::InstallService { name } => {
//             let installer = Installer::new(name, base_dir, config, api);
//             installer.install_service().await?
//         }
//     }
//
//     Ok(())
// }

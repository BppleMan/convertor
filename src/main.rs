mod cli;

use crate::cli::{ConvertorCli, ConvertorCommand};
use clap::Parser;
use color_eyre::Result;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::install_service::Installer;
use convertor::router::start_server;
use convertor::service_provider::SubscriptionService;
use convertor::service_provider::subscription_api::boslife_api::BosLifeApi;
use convertor::{init_backtrace, init_base_dir, init_log};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(&base_dir);

    let cli = ConvertorCli::parse();
    let config = ConvertorConfig::search(&base_dir, cli.config)?;
    let client = reqwest::Client::new();
    let api = BosLifeApi::new(&base_dir, client, config.service_config.clone());

    match cli.command {
        None => start_server(cli.listen, config, api, &base_dir).await?,
        Some(ConvertorCommand::Subscription(args)) => {
            let mut subscription_service = SubscriptionService { config, api };
            subscription_service.execute(*args).await?;
        }
        Some(ConvertorCommand::InstallService { name }) => {
            let installer = Installer::new(name, base_dir, config, api);
            installer.install_service().await?
        }
    }

    Ok(())
}

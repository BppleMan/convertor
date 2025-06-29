mod cli;

use crate::cli::{ConvertorCli, ConvertorCommand};
use clap::Parser;
use color_eyre::Result;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::install_service::install_service;
use convertor::server::start_server;
use convertor::subscription::subscription_api::boslife_api::BosLifeApi;
use convertor::subscription::subscription_service::SubscriptionService;
use convertor::{base_dir, init};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let base_dir = base_dir();
    init(&base_dir);

    let cli = ConvertorCli::parse();
    let config = ConvertorConfig::search(&base_dir, cli.config)?;
    let client = reqwest::Client::new();
    let service = BosLifeApi::new(client, config.service_config.clone());

    match cli.command {
        None => start_server(cli.listen, config, service, base_dir).await?,
        Some(ConvertorCommand::Subscription { command, server, flag }) => {
            let subscription_service = SubscriptionService { config };
            let server = server.map(|s| Url::parse(&s)).transpose()?;
            subscription_service.execute(command, server, flag).await?;
        }
        Some(ConvertorCommand::InstallService { name }) => install_service(name, service).await?,
    }

    Ok(())
}

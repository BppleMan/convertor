use clap::Parser;
use color_eyre::Result;
use convertor::boslife::boslife_service::BosLifeService;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::server::start_server;
use convertor::{base_dir, init, ConvertorCli, Service};

#[tokio::main]
async fn main() -> Result<()> {
    let base_dir = base_dir();
    init(&base_dir);

    let config = ConvertorConfig::search(&base_dir)?;
    let cli = ConvertorCli::parse();
    let client = reqwest::Client::new();
    let service = BosLifeService::new(client, config);

    match cli.command {
        None => start_server(cli, service, base_dir).await?,
        Some(Service::BosLife {
            command,
            server,
            flag,
        }) => {
            service.execute(command, server, flag).await?;
        }
        Some(Service::InstallService { name }) => {}
    }

    Ok(())
}

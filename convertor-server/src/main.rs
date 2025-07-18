use clap::Parser;
use color_eyre::Result;
use convertor::server::start_server;
use convertor_core::api::ServiceApi;
use convertor_core::config::ConvertorConfig;
use convertor_core::{init_backtrace, init_base_dir, init_log};
use std::net::SocketAddrV4;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, author)]
pub struct ConvertorCli {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8001")]
    pub listen: SocketAddrV4,

    #[arg(short)]
    pub config: Option<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(&base_dir);

    let cli = ConvertorCli::parse();
    let config = ConvertorConfig::search(&base_dir, cli.config)?;
    let client = reqwest::Client::new();
    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir, client);

    start_server(cli.listen, config, api, &base_dir).await?;

    Ok(())
}

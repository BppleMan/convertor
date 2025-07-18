use clap::{Parser, Subcommand};
use convertor::install_service::ServiceName;
use convertor::service_provider::args::ServiceProviderArgs;
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

    #[command(subcommand)]
    pub command: Option<ConvertorCommand>,
}

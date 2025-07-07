use clap::{Parser, Subcommand};
use convertor::install_service::ServiceName;
use convertor::subscription::subscription_args::SubscriptionArgs;
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

#[derive(Debug, Subcommand)]
pub enum ConvertorCommand {
    /// 服务商订阅配置
    #[command(name = "sub")]
    Subscription(Box<SubscriptionArgs>),

    /// 安装服务
    #[command(name = "install")]
    InstallService {
        /// 服务名称
        #[arg(value_enum, default_value = "convertor")]
        name: ServiceName,
    },
}

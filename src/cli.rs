use clap::{Parser, Subcommand};
use convertor::subscription::subscription_command::SubscriptionCommand;
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
    /// 操作 boslife 订阅配置
    #[command(name = "sub")]
    Subscription {
        #[command(subcommand)]
        command: SubscriptionCommand,

        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, long)]
        server: Option<String>,

        /// 构造适用于 surge/clash 的订阅地址
        #[arg()]
        flag: String,
    },

    /// 安装服务
    #[command(name = "install")]
    InstallService {
        /// 服务名称 [mihomo, convertor]
        #[arg(default_value = "convertor")]
        name: String,
    },
}

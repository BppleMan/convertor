use crate::cli::config_cli::ConfigCmd;
use crate::cli::subscription_cli::SubscriptionCmd;
use clap::Subcommand;

pub mod config_cli;
pub mod subscription_cli;
pub mod update;

#[derive(Debug, Clone, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConvertorCommand {
    /// 配置相关的子命令
    /// 获取配置模板, 生成配置文件等
    #[command(name = "config")]
    Config(ConfigCmd),

    /// 获取订阅提供商的订阅链接
    #[command(name = "sub")]
    Subscription(SubscriptionCmd),
}

use crate::cli::service_installer::ServiceName;
use crate::cli::sub_provider_executor::SubProviderCmd;
use clap::Subcommand;

pub mod service_installer;
pub mod sub_provider_executor;

#[derive(Debug, Clone, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConvertorCommand {
    /// 获取订阅提供商的订阅链接
    #[command(name = "sub")]
    Subscription(SubProviderCmd),

    /// 安装 systemd 服务
    #[command(name = "install")]
    Install {
        /// 服务名称
        #[arg(value_enum, default_value = "convertor")]
        name: ServiceName,
    },
}

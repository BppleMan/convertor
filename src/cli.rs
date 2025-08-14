use crate::cli::service_installer::ServiceName;
use crate::cli::sub_provider_executor::SubProviderCmd;
use crate::common::config::config_cmd::ConfigCmd;
use crate::common::config::sub_provider::SubProvider;
use clap::Subcommand;

pub mod service_installer;
pub mod sub_provider_executor;

#[derive(Debug, Clone, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConvertorCommand {
    /// 配置相关的子命令
    /// 获取配置模板, 生成配置文件等
    #[command(name = "config")]
    Config(ConfigCmd),

    /// 获取订阅提供商的订阅链接
    #[command(name = "sub")]
    SubProvider(SubProviderCmd),

    /// 安装 systemd 服务
    #[command(name = "install")]
    Install {
        /// 服务名称
        #[arg(value_enum, default_value_t = ServiceName::Convertor)]
        name: ServiceName,

        /// 订阅提供商
        #[arg(value_enum, default_value_t = SubProvider::BosLife)]
        provider: SubProvider,
    },
}

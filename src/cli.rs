use crate::cli::install_service::ServiceName;
use crate::cli::service_provider::ServiceProviderArgs;
use clap::Parser;

pub mod install_service;
pub mod service_provider;

#[derive(Debug, Parser)]
#[allow(clippy::large_enum_variant)]
pub enum ConvertorCli {
    /// 服务商订阅配置
    #[command(name = "sub")]
    Subscription(ServiceProviderArgs),

    /// 安装服务
    #[command(name = "install")]
    InstallService {
        /// 服务名称
        #[arg(value_enum, default_value = "convertor")]
        name: ServiceName,
    },
}

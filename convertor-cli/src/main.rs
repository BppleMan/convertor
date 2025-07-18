mod install_service;
pub mod service_provider;

use crate::install_service::{Installer, ServiceName};
use crate::service_provider::{ServiceProviderArgs, SubscriptionService};
use clap::Parser;
use color_eyre::Report;
use convertor_core::api::ServiceApi;
use convertor_core::config::ConvertorConfig;
use convertor_core::{init_backtrace, init_base_dir, init_log};

#[derive(Debug, Parser)]
pub enum ConvertorCli {
    /// 服务商订阅配置
    #[command(name = "sub")]
    Subscription(Box<ServiceProviderArgs>),

    /// 安装服务
    #[command(name = "install")]
    InstallService {
        /// 服务名称
        #[arg(value_enum, default_value = "convertor")]
        name: ServiceName,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Report> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(&base_dir);

    let cli = ConvertorCli::parse();
    let config = ConvertorConfig::search(&base_dir, None::<&str>)?;
    let client = reqwest::Client::new();
    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir, client);

    match cli {
        ConvertorCli::Subscription(args) => {
            let mut subscription_service = SubscriptionService { config, api };
            subscription_service.execute(*args).await?;
        }
        ConvertorCli::InstallService { name } => {
            let installer = Installer::new(name, base_dir, config, api);
            installer.install_service().await?
        }
    }

    Ok(())
}

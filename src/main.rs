use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use convertor::api::SubProviderWrapper;
use convertor::cli::ConvertorCommand;
use convertor::cli::service_installer::ServiceInstaller;
use convertor::cli::sub_provider_executor::SubProviderExecutor;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::config_cmd::ConfigCmdExecutor;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::common::redis_info::{config_center_url, init_redis_info};
use convertor::server::start_server;
use std::net::SocketAddrV4;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, author)]
/// 启动 Convertor 服务
pub struct Convertor {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8080")]
    listen: SocketAddrV4,

    /// 如果你想特别指定配置文件, 可以使用此参数
    #[arg(short)]
    config: Option<PathBuf>,

    /// 对于启动 Convertor 服务, 子命令不是必须的, 子命令仅作为一次性执行指令
    #[command(subcommand)]
    command: Option<ConvertorCommand>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(&base_dir);
    init_redis_info()?;

    let mut args = Convertor::parse();
    match args.command.take() {
        Some(ConvertorCommand::Config(config_cmd)) => {
            ConfigCmdExecutor::new(config_cmd).execute().await?;
        }
        other => {
            let redis_client = redis::Client::open(config_center_url())?;
            let connection = redis_client.get_multiplexed_async_connection().await?;
            let config = ConvertorConfig::search_or_redis(&base_dir, args.config, connection).await?;
            let mut api_map = SubProviderWrapper::create_api(config.providers.clone(), &base_dir);

            match other {
                None => start_server(args.listen, config, api_map, &base_dir).await?,
                Some(ConvertorCommand::SubProvider(args)) => {
                    let mut executor = SubProviderExecutor::new(config, api_map);
                    let (url_builder, result) = executor.execute(args).await?;
                    executor.post_execute(url_builder, result);
                }
                Some(ConvertorCommand::Install { name, provider }) => {
                    let Some(api) = api_map.remove(&provider) else {
                        return Err(eyre!("没有找到对应的订阅提供者: {provider}"));
                    };
                    ServiceInstaller::new(name, base_dir, config, api).install().await?
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

use clap::Parser;
use color_eyre::Result;
use convertor::api::UniversalProviderApi;
use convertor::cli::ConvertorCommand;
use convertor::cli::service_installer::ServiceInstaller;
use convertor::cli::sub_provider_executor::SubProviderExecutor;
use convertor::common::config::ConvertorConfig;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::server::start_server;
use std::net::SocketAddrV4;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, author)]
/// 启动 Convertor 服务
pub struct Convertor {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8001")]
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

    let args = Convertor::parse();
    let config = ConvertorConfig::search(&base_dir, args.config)?;
    let api = UniversalProviderApi::get_service_provider_api(config.provider.clone(), &base_dir);

    match args.command {
        None => start_server(args.listen, config, api, &base_dir).await?,
        Some(ConvertorCommand::Subscription(args)) => {
            let result = SubProviderExecutor::new(config, api).execute(args).await?;
            println!("{result}");
        }
        Some(ConvertorCommand::Install { name }) => {
            ServiceInstaller::new(name, base_dir, config, api).install().await?
        }
    }

    Ok(())
}

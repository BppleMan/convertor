use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use color_eyre::Result;
use convertor::cli::ConvertorCommand;
use convertor::cli::config_cli::ConfigCli;
use convertor::cli::provider_cli::ProviderCli;
use convertor::common::config::ConvertorConfig;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::common::redis::{init_redis, redis_client, redis_url};
use convertor::provider_api::ProviderApi;
use convertor::server::start_server;
use std::net::SocketAddrV4;
use std::path::PathBuf;

// 定义一个静态的 Styles 常量，设置我们想要的颜色样式
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold()) // 一级标题（如 "Usage:", "Options:"）使用黄色粗体
    .usage(AnsiColor::Green.on_default().bold()) // Usage 行使用绿色粗体
    .literal(AnsiColor::Blue.on_default().bold()) // 字面量文本（比如命令名、旗标）使用蓝色粗体
    .placeholder(AnsiColor::Cyan.on_default()); // 占位符（比如 <参数>）使用青色

#[derive(Debug, Parser)]
#[clap(version, author, styles = STYLES)]
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
    init_log(Some(&base_dir));
    init_redis()?;

    let mut args = Convertor::parse();
    let redis_client = redis_client(redis_url())?;
    match args.command.take() {
        Some(ConvertorCommand::Config(config_cmd)) => {
            ConfigCli::new(config_cmd).execute(redis_client).await?;
        }
        other => {
            let connection = redis_client.get_multiplexed_async_connection().await?;
            let connection_manager = redis::aio::ConnectionManager::new_with_config(
                redis_client.clone(),
                redis::aio::ConnectionManagerConfig::new()
                    .set_number_of_retries(5)
                    .set_max_delay(2000),
            )
            .await?;
            let config = ConvertorConfig::search_or_redis(&base_dir, args.config, connection).await?;
            let api_map = ProviderApi::create_api(config.providers.clone(), connection_manager);

            match other {
                None => start_server(args.listen, config, api_map, &base_dir, redis_client).await?,
                Some(ConvertorCommand::SubProvider(args)) => {
                    let mut executor = ProviderCli::new(config, api_map);
                    let (url_builder, result) = executor.execute(args).await?;
                    executor.post_execute(url_builder, result);
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

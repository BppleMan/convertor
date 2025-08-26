use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::RgbColor;
use color_eyre::Result;
use confly::cli::ConvertorCommand;
use confly::cli::config_cli::ConfigCli;
use confly::cli::provider_cli::ProviderCli;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::common::redis::{init_redis, redis_client, redis_url};
use convertor::config::ConvertorConfig;
use convertor::provider_api::ProviderApi;
use std::path::PathBuf;

// Sonokai · Truecolor（24-bit，更贴近主题）
// pub const SONOKAI: Styles = Styles::styled()
//     .header(RgbColor(0xFD, 0x97, 0x1F).on_default().bold().underline())
//     .usage(RgbColor(0xA6, 0xE2, 0x2E).on_default().bold())
//     .literal(RgbColor(0xF9, 0x26, 0x72).on_default().bold())
//     .placeholder(RgbColor(0x66, 0xD9, 0xEF).on_default().italic().dimmed())
//     .error(RgbColor(0xFF, 0x55, 0x55).on_default().bold())
//     .valid(RgbColor(0xA6, 0xE2, 0x2E).on_default().bold())
//     .invalid(RgbColor(0xFF, 0x55, 0x55).on_default());

pub const SONOKAI_TC: Styles = Styles::styled()
    // “Usage: / Options:” 标题：yellow，粗体+下划线
    .header(RgbColor(0xE7, 0xC6, 0x64).on_default().bold().underline())
    // Usage 行：green，粗体
    .usage(RgbColor(0x9E, 0xD0, 0x72).on_default().bold())
    // 字面量（命令/旗标）：orange，粗体
    .literal(RgbColor(0xF3, 0x96, 0x60).on_default().bold())
    // 占位符（<ARG>）：blue，斜体+弱化
    .placeholder(RgbColor(0x76, 0xCC, 0xE0).on_default().italic().dimmed())
    // 错误：red，粗体
    .error(RgbColor(0xFC, 0x5D, 0x7C).on_default().bold())
    // 校验通过/失败
    .valid(RgbColor(0x9E, 0xD0, 0x72).on_default().bold())
    .invalid(RgbColor(0xFC, 0x5D, 0x7C).on_default());

#[derive(Debug, Parser)]
#[clap(version, author, styles = SONOKAI_TC)]
/// 启动 Convertor 服务
pub struct Convertor {
    /// 对于启动 Convertor 服务, 子命令不是必须的, 子命令仅作为一次性执行指令
    #[command(subcommand)]
    command: ConvertorCommand,

    /// 如果你想特别指定配置文件, 可以使用此参数
    #[arg(short)]
    config: Option<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();
    init_log(Some(&base_dir));
    init_redis()?;

    let args = Convertor::parse();
    let redis_client = redis_client(redis_url())?;
    match args.command {
        ConvertorCommand::Config(config_cmd) => {
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
                ConvertorCommand::SubProvider(args) => {
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

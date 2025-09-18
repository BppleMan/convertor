use clap::Parser;
use color_eyre::Result;
use confly::cli::config_cli::ConfigCli;
use confly::cli::provider_cli::ProviderCli;
use confly::cli::ConvertorCommand;
use convertor::common::clap_style::SONOKAI_TC;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::config::ConvertorConfig;
use convertor::provider_api::ProviderApi;
use std::path::PathBuf;

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
    let args = Convertor::parse();

    let base_dir = init_base_dir();
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    init_log(Some(&base_dir));

    match args.command {
        ConvertorCommand::Config(config_cmd) => {
            ConfigCli::new(config_cmd).execute().await?;
        }
        other => {
            let config = ConvertorConfig::search(&base_dir, args.config)?;
            let api_map = ProviderApi::create_api_no_redis(config.providers.clone());

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

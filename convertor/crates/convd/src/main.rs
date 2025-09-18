use clap::{Parser, Subcommand};
use color_eyre::Result;
use convd::server::start_server;
use convertor::common::clap_style::SONOKAI_TC;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::config::ConvertorConfig;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Parser)]
#[clap(version, author, styles = SONOKAI_TC)]
/// 启动 Convertor 服务
struct Convd {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8080")]
    listen: SocketAddrV4,

    /// 如果你想特别指定配置文件, 可以使用此参数
    #[arg(short)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    sub_cmd: SubCmd,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    Tag,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Convd::parse();

    let base_dir = init_base_dir();
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    init_log(Some(&base_dir));
    info!("工作目录: {}", base_dir.display());

    info!("+──────────────────────────────────────────────+");
    info!("│               加载配置文件...                │");
    info!("+──────────────────────────────────────────────+");
    let config = ConvertorConfig::search(&base_dir, args.config)?;
    info!("配置文件加载完成");

    start_server(args.listen, config).await?;

    Ok(())
}

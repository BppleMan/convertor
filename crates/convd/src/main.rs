use clap::{Parser, Subcommand};
use color_eyre::Result;
use convd::server::start_server;
use convertor::common::clap_style::SONOKAI_TC;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::config::Config;
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
    sub_cmd: Option<SubCmd>,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    Tag,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Convd::parse();
    if let Some(SubCmd::Tag) = args.sub_cmd {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let base_dir = init_base_dir();
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    let loki_url = std::env::var("LOKI_URL").ok();
    let otlp_grpc = std::env::var("OTLP_GRPC").ok();
    let loki_task = init_log(loki_url.as_deref(), otlp_grpc.as_deref());

    // 启动 loki 后台任务
    if let Some(task) = loki_task {
        tokio::spawn(task);
    }

    info!("工作目录: {}", base_dir.display());

    info!("+──────────────────────────────────────────────+");
    info!("│               加载配置文件...                │");
    info!("+──────────────────────────────────────────────+");
    let config: Config = Config::search(&base_dir, args.config)?;
    info!("配置文件加载完成");

    start_server(args.listen, config).await?;

    Ok(())
}

use clap::Parser;
use color_eyre::Result;
use convd::server::start_server;
use convertor::common::clap_style::SONOKAI_TC;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::common::redis::{init_redis, redis_client, redis_url};
use convertor::config::ConvertorConfig;
use convertor::provider_api::ProviderApi;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use tracing::{debug, info};

#[derive(Debug, Parser)]
#[clap(version, author, styles = SONOKAI_TC)]
/// 启动 Convertor 服务
pub struct Convertor {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8080")]
    listen: SocketAddrV4,

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
    info!("工作目录: {}", base_dir.display());

    let args = Convertor::parse();
    info!("+──────────────────────────────────────────────+");
    info!("│             初始化 Redis 连接...             │");
    info!("+──────────────────────────────────────────────+");
    let redis_client = redis_client(redis_url())?;
    debug!("等待 multiplexed connection 就绪...");
    let connection = redis_client.get_multiplexed_async_connection().await?;
    debug!("等待 connection_manager 就绪...");
    let connection_manager = redis::aio::ConnectionManager::new_with_config(
        redis_client.clone(),
        redis::aio::ConnectionManagerConfig::new()
            .set_number_of_retries(5)
            .set_max_delay(2000),
    )
    .await?;
    info!("Redis 连接就绪");

    info!("+──────────────────────────────────────────────+");
    info!("│               加载配置文件...                │");
    info!("+──────────────────────────────────────────────+");
    let config = ConvertorConfig::search_or_redis(&base_dir, args.config, connection).await?;
    let api_map = ProviderApi::create_api(config.providers.clone(), connection_manager);
    info!("配置文件加载完成");

    info!("+──────────────────────────────────────────────+");
    info!("│                 启动服务...                  │");
    info!("+──────────────────────────────────────────────+");
    start_server(args.listen, config, api_map, redis_client).await?;

    Ok(())
}

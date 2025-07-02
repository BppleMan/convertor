use crate::config::convertor_config::ConvertorConfig;
use crate::server::router::{router, AppState};
use crate::shutdown_signal;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use std::net::SocketAddrV4;
use std::path::Path;
use tracing::{info, warn};

pub mod router;

pub async fn start_server(
    listen_addr: SocketAddrV4,
    convertor_config: ConvertorConfig,
    subscription_api: BosLifeApi,
    base_dir: impl AsRef<Path>,
) -> color_eyre::Result<()> {
    info!("base_dir: {}", base_dir.as_ref().display());
    info!("监听中: {}", &listen_addr);
    warn!("建议使用 nginx 等网关进行反向代理，以开启 HTTPS 支持");
    info!("服务启动，使用 Ctrl+C 或 SIGTERM 关闭服务");
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    let app_state = AppState {
        convertor_config,
        subscription_api,
    };
    axum::serve(listener, router(app_state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("服务关闭");
    Ok(())
}

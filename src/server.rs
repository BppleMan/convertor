use crate::api::SubProviderWrapper;
use crate::common::config::ConvertorConfig;
use crate::common::config::sub_provider::SubProvider;
use crate::server::app_state::AppState;
use color_eyre::Result;
use std::collections::HashMap;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::Path;
use tokio::signal;
use tracing::{info, warn};

pub mod router;
pub mod error;
pub mod surge_service;
pub mod clash_service;
pub mod raw_service;
pub mod app_state;
pub mod query;

pub async fn start_server(
    listen_addr: SocketAddrV4,
    config: ConvertorConfig,
    api_map: HashMap<SubProvider, SubProviderWrapper>,
    base_dir: impl AsRef<Path>,
) -> Result<()> {
    info!("base_dir: {}", base_dir.as_ref().display());
    info!("监听中: {}", &listen_addr);
    warn!("建议使用 nginx 等网关进行反向代理，以开启 HTTPS 支持");
    info!("服务启动，使用 Ctrl+C 或 SIGTERM 关闭服务");
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    let app_state = AppState::new(config, api_map);
    let app = router::router(app_state);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("服务关闭");
    Ok(())
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

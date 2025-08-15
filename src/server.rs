use crate::api::SubProviderWrapper;
use crate::common::config::ConvertorConfig;
use crate::common::config::sub_provider::SubProvider;
use crate::common::ext::WatchDebounceExt;
use crate::common::redis_info::REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL;
use crate::server::app_state::AppState;
use axum::Router;
use color_eyre::eyre::WrapErr;
use color_eyre::{Report, Result};
use std::collections::HashMap;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::Path;
use tokio::net::{TcpListener, TcpSocket};
use tokio::signal;
use tokio::sync::watch;
use tokio::sync::watch::Receiver;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

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
    client: redis::Client,
) -> Result<()> {
    info!("工作环境: {}", base_dir.as_ref().display());
    info!("监听中: {}", &listen_addr);
    warn!("建议使用 nginx 等网关进行反向代理，以开启 HTTPS 支持");

    let cancel_token = CancellationToken::new();
    let mut config_receiver = start_sub_config(client, config);
    let mut current_config = config_receiver.borrow().clone();

    loop {
        // 1) 启一个子 token 控制“本轮” server
        let stop_this = cancel_token.child_token();

        // 2) 构建 state / router
        let state = AppState::new(current_config, api_map.clone());
        let app: Router = router::router(state);

        // 3) 绑定端口（Tokio 自带 TcpSocket，可设置 reuseaddr；不引新库）
        let listener = bind_once(listen_addr)?;

        // 4) 跑 server，直到被 cancel
        let serve_handle = tokio::spawn({
            let stop_this = stop_this.clone();
            async move {
                info!("服务启动，使用 Ctrl+C 或 SIGTERM 关闭服务");
                axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                    .with_graceful_shutdown(async move { stop_this.cancelled().await })
                    .await
            }
        });

        // 5) 等待：要么全局退出，要么收到“去抖后的配置更新”
        tokio::select! {
            _ = shutdown_signal() => {
                // 收到 Ctrl+C 或 SIGTERM 信号，准备退出
                info!("收到退出信号，准备关闭服务…");
                stop_this.cancel();
                let _ = serve_handle.await;
                break;
            }
            _ = cancel_token.cancelled() => {
                // 全局退出：关本轮 server 并退出
                stop_this.cancel();
                let _ = serve_handle.await;
                break;
            }
            Ok(config) = config_receiver.recv_debounced_distinct(tokio::time::Duration::from_secs(1), |_| true) => {
                info!("收到配置更新通知，准备重启服务…");
                info!("停止旧的服务…");
                stop_this.cancel();
                let _ = serve_handle.await;
                info!("更新配置…");
                current_config = config.clone();
                continue;
            }
        }
    }
    info!("服务关闭");
    Ok(())
}

fn bind_once(addr: SocketAddrV4) -> Result<TcpListener> {
    let sock = TcpSocket::new_v4()?;
    sock.set_reuseaddr(true)?;
    // #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
    // sock.set_reuseport(true)?; // 可选：看需要
    sock.bind(SocketAddr::V4(addr))?;
    Ok(sock.listen(1024)?)
}

// 启动 ConvertorConfig 的更新订阅
fn start_sub_config(client: redis::Client, config: ConvertorConfig) -> Receiver<ConvertorConfig> {
    let (tx, rx) = watch::channel(config);
    tokio::spawn(async move {
        info!("启动 Redis subscribe");
        let connection = client.get_multiplexed_async_connection().await?;
        let mut sub = client.get_async_pubsub().await?;
        sub.subscribe(REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL).await?;
        let mut stream = sub.into_on_message();
        while let Some(msg) = stream.next().await {
            info!("Redis sub 接收到: {msg:?}");
            if msg.get_channel_name() == REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL {
                if let Err(e) = ConvertorConfig::from_redis(connection.clone())
                    .await
                    .and_then(|config| tx.send(config).wrap_err("无法发送更新配置"))
                {
                    error!("{e:?}");
                }
            }
        }
        Ok::<(), Report>(())
    });
    rx
}

async fn shutdown_signal() {
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

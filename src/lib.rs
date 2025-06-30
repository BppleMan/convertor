use std::path::{Path, PathBuf};
use tokio::signal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub mod error;
pub mod profile;
pub mod region;
pub mod encrypt;
pub mod config;
pub mod server;
pub mod install_service;
pub mod subscription;

pub fn base_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap();
    #[cfg(not(debug_assertions))]
    let base_dir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())).join(".convertor");
    base_dir
}

pub fn init_backtrace() {
    if let Err(e) = color_eyre::install() {
        eprintln!("Failed to install color_eyre: {}", e);
    }
}

pub fn init_log(base_dir: impl AsRef<Path>) {
    let filter = EnvFilter::new("info")
        .add_directive("convertor=trace".parse().unwrap())
        .add_directive("tower_http=trace".parse().unwrap())
        .add_directive("moka=trace".parse().unwrap());

    let file_appender = tracing_appender::rolling::hourly(base_dir.as_ref().join("logs"), "convertor.log");

    // let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer().with_writer(file_appender);

    let stdout_layer = tracing_subscriber::fmt::layer();
    // .pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();
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

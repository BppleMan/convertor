use crate::boslife::boslife_command::BosLifeCommand;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tokio::signal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub mod error;
pub mod profile;
pub mod region;
pub mod encrypt;
pub mod service;
pub mod convertor_url;
pub mod config;
pub mod boslife;
pub mod server;

#[derive(Debug, Parser)]
#[clap(version, author)]
pub struct ConvertorCli {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8001")]
    pub listen: String,

    #[command(subcommand)]
    pub command: Option<Service>,
}

#[derive(Debug, Subcommand)]
pub enum Service {
    /// 操作 boslife 订阅配置
    #[command(name = "bl")]
    BosLife {
        #[command(subcommand)]
        command: BosLifeCommand,
    },
}

pub fn base_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap();
    #[cfg(not(debug_assertions))]
    let base_dir = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
    )
    .join(".convertor");
    base_dir
}

pub fn init<P: AsRef<Path>>(base_dir: P) {
    color_eyre::install().unwrap();

    let filter = EnvFilter::new("info")
        .add_directive("convertor=trace".parse().unwrap())
        .add_directive("tower_http=trace".parse().unwrap())
        .add_directive("moka=trace".parse().unwrap());

    let file_appender = tracing_appender::rolling::hourly(
        base_dir.as_ref().join("logs"),
        "convertor.log",
    );

    // let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer =
        tracing_subscriber::fmt::layer().with_writer(file_appender);

    let stdout_layer = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
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

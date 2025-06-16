use crate::route::AppState;
use crate::service::boslife_service::{
    BosLifeService, BosLifeServiceSubscription,
};
use axum::routing::get;
use axum::Router;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::signal;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod error;
mod profile;
mod route;
mod region;
mod middleware;
mod op;
mod encrypt;
mod service;
mod convertor_url;
mod config;

#[derive(Debug, Parser)]
#[clap(version, author)]
struct ConvertorCli {
    /// 监听地址, 不需要指定协议
    #[arg(default_value = "127.0.0.1:8001")]
    listen: String,

    #[command(subcommand)]
    command: Option<Service>,
}

#[derive(Debug, Subcommand)]
enum Service {
    /// 操作 boslife 订阅配置
    #[command(name = "bl")]
    BosLife {
        #[command(subcommand)]
        operation: BosLifeServiceSubscription,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_dir = base_dir();
    init(&base_dir);

    let cli = ConvertorCli::parse();
    let client = reqwest::Client::new();
    let service = BosLifeService::new(client);

    match cli.command {
        None => start_server(cli, AppState { service }, base_dir).await?,
        Some(Service::BosLife { operation }) => {
            operation.execute(service).await?;
        }
    }

    Ok(())
}

async fn start_server(
    cli: ConvertorCli,
    app_state: AppState,
    base_dir: PathBuf,
) -> Result<()> {
    info!("{:#?}", cli);
    info!("base_dir: {:?}", base_dir);

    let app = Router::new()
        .route("/", get(route::root))
        .route("/clash", get(route::clash::profile))
        .route("/surge", get(route::surge::profile))
        .route("/surge/rule_set", get(route::surge::rule_set))
        .route(
            "/subscription_log",
            get(route::subscription::subscription_log),
        )
        .with_state(Arc::new(app_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    // let addr = format!("{}:{}", cli.host, cli.port);
    let addr = cli.listen;

    info!("Listening on: http://{}", addr);
    warn!("It is recommended to use nginx for reverse proxy and enable SSL");
    info!("usage: all url parameters need to be url-encoded");
    info!(
        "\tmain sub: http://{}/surge?url=[boslife subscription url]",
        addr
    );
    info!(
        "\trule set: http://{}/rule-set?url=[boslife subscription url]&boslife=true",
        addr
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server starting");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("Server stopped");
    Ok(())
}

fn base_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap();
    #[cfg(not(debug_assertions))]
    let base_dir = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
    )
    .join(".convertor");
    base_dir
}

fn init<P: AsRef<Path>>(base_dir: P) {
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

async fn shutdown_signal() {
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

use std::path::{Path, PathBuf};

use crate::boslife::update_profile;
use axum::routing::get;
use axum::Router;
use clap::{Args, Parser, Subcommand};
use color_eyre::Result;
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
mod boslife;
mod op;
mod encrypt;

// a clap command line argument parser
#[derive(Debug, Parser)]
#[clap(version, author)]
struct Convertor {
    #[clap(flatten)]
    common: CommonArgs,

    #[command(subcommand)]
    sub_cmd: Option<SubCmd>,
}

#[derive(Debug, Subcommand)]
enum SubCmd {
    UpdateProfile(UpdateProfile),
}

#[derive(Debug, Args)]
struct UpdateProfile {
    #[clap(flatten)]
    common: CommonArgs,
    #[arg(short, long, default_value = "false")]
    refresh_token: bool,
}

#[derive(Debug, Clone, Args)]
struct CommonArgs {
    /// host to listen on
    #[arg(long, default_value = "http://127.0.0.1")]
    host: String,

    /// port to listen on
    #[arg(long, default_value = "8001")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_dir = base_dir();
    init(&base_dir);

    let convertor = Convertor::parse();

    if let Some(SubCmd::UpdateProfile(UpdateProfile {
        common,
        refresh_token,
    })) = convertor.sub_cmd
    {
        let addr = format!("{}:{}", common.host, common.port);
        update_profile(&addr, refresh_token).await?;
    } else {
        info!("{:#?}", convertor);
        info!("base_dir: {:?}", base_dir);

        let app = Router::new()
            .route("/", get(route::root))
            .route("/clash", get(route::clash::profile))
            .route("/surge", get(route::surge::profile))
            .route("/surge/rule_set", get(route::surge::rule_set))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        DefaultMakeSpan::new().include_headers(true),
                    )
                    .on_request(
                        DefaultOnRequest::new().level(tracing::Level::INFO),
                    )
                    .on_response(
                        DefaultOnResponse::new()
                            .level(tracing::Level::INFO)
                            .latency_unit(LatencyUnit::Millis),
                    ),
            );

        let addr =
            format!("{}:{}", convertor.common.host, convertor.common.port);

        info!("Listening on: http://{}", addr);
        warn!(
            "It is recommended to use nginx for reverse proxy and enable SSL"
        );
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
    }

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
        .add_directive("tower_http=trace".parse().unwrap());

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

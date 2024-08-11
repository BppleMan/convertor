use axum::routing::get;
use axum::Router;
use clap::Parser;
use color_eyre::Result;
use tokio::signal;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod error;
mod profile;
mod route;
mod region;
mod middleware;

pub const MOCK_CONTENT: &str = include_str!("../assets/mock");

// a clap command line argument parser
#[derive(Debug, Parser)]
#[clap(name = "convertor", version, author)]
struct Convertor {
    #[clap(short, long, default_value = "8196")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let convertor = Convertor::parse();

    let _guard = init();

    let app = Router::new()
        .route("/", get(route::root))
        .route("/clash", get(route::clash::profile))
        .route("/surge", get(route::surge::profile))
        .route("/surge/rule_set", get(route::surge::rule_set))
        .route("/api/v1/client/subscribe", get(MOCK_CONTENT))
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

    let addr = format!("0.0.0.0:{}", convertor.port);

    println!("Listening on: http://{}", addr);
    println!("usage: all url parameters need to be url-encoded");
    println!(
        "\tmain sub: http://{}/surge?url=[boslife subscription url]",
        addr
    );
    println!(
        "\trule set: http://{}/rule-set?url=[boslife subscription url]&boslife=true",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init() -> WorkerGuard {
    color_eyre::install().unwrap();

    let filter = EnvFilter::new("info")
        .add_directive("convertor=trace".parse().unwrap())
        .add_directive("tower_http=trace".parse().unwrap());

    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap();
    #[cfg(not(debug_assertions))]
    let base_dir = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
    )
    .join(".convertor");
    let file_appender = tracing_appender::rolling::hourly(
        base_dir.join("logs"),
        "convertor.log",
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer().with_writer(non_blocking);

    let stdout_layer = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    guard
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

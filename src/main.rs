mod error;
mod profile;
mod route;

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = init();

    let app = Router::new()
        .route("/", get(route::root))
        .route("/clash", get(route::clash::profile))
        .route("/surge", get(route::surge::profile))
        .route("/rule_set", get(route::surge::rule_set));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8196").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

fn init() -> WorkerGuard {
    let filter = EnvFilter::new("info").add_directive("convertor=trace".parse().unwrap());

    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap();
    #[cfg(not(debug_assertions))]
    let base_dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()))
        .join(".convertor");
    let file_appender = tracing_appender::rolling::hourly(base_dir.join("logs"), "convertor.log");
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

mod error;
mod profile;
mod route;
mod region;
mod middleware;

use anyhow::Result;
use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use tokio::task::JoinHandle;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub const MOCK_CONTENT: &str = include_str!("../assets/mock");

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = init();

    let app = Router::new()
        .route("/", get(route::root))
        .route("/clash", get(route::clash::profile))
        .route("/surge", get(route::surge::profile))
        .route("/surge/rule_set", get(route::surge::rule_set))
        .route("/api/v1/client/subscribe", get(MOCK_CONTENT));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8196").await?;

    let _handle: JoinHandle<Result<()>> = tokio::spawn(async move {
        axum::serve(listener, app).await?;
        Ok(())
    });

    let rule_set = route::surge::rule_set(Query(route::surge::SurgeQuery {
        url: "http://127.0.0.1:8196/api/v1/client/subscribe?token=4287df4ba2618c4da1f9efd7ffeb30a5".to_string(),
        flag: "surge".to_string(),
        policies: Some("DIRECT".to_string()),
    }))
    .await?;

    println!("rule-set:\n{}", rule_set);
    _handle.await??;

    Ok(())
}

fn init() -> WorkerGuard {
    let filter = EnvFilter::new("info").add_directive("convertor=trace".parse().unwrap());

    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().unwrap();
    #[cfg(not(debug_assertions))]
    let base_dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())).join(".convertor");
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

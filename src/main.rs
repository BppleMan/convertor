mod error;
mod profile;
mod route;
mod region;
mod middleware;

use axum::body::Bytes;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use color_eyre::Result;
use http_body_util::BodyExt;
use tracing::debug;
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
        .route("/api/v1/client/subscribe", get(MOCK_CONTENT))
        .layer(axum::middleware::from_fn(print_request_response));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8196").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

fn init() -> WorkerGuard {
    color_eyre::install().unwrap();

    let filter = EnvFilter::new("info")
        .add_directive("convertor=trace".parse().unwrap());

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

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    debug!("{:#?}", req);
    // let (parts, body) = req.into_parts();
    // let bytes = buffer_and_print("request", body).await?;
    // let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    debug!("{:#?}", res);
    // let (parts, body) = res.into_parts();
    // let bytes = buffer_and_print("response", body).await?;
    // let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn _buffer_and_print<B>(
    direction: &str,
    body: B,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        debug!("{direction} body = {body}");
    }

    Ok(bytes)
}

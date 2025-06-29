use crate::config::convertor_config::ConvertorConfig;
use crate::server::route::AppState;
use crate::shutdown_signal;
use crate::subscription::subscription_api::boslife_api::BosLifeApi;
use axum::routing::get;
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{info, warn};

pub mod route;

pub async fn start_server(
    listen_addr: impl AsRef<str>,
    convertor_config: ConvertorConfig,
    subscription_api: BosLifeApi,
    base_dir: PathBuf,
) -> color_eyre::Result<()> {
    info!("base_dir: {:?}", base_dir);

    let app_state = AppState {
        convertor_config,
        subscription_api,
    };
    let app = Router::new()
        .route("/", get(route::root))
        .route("/surge", get(route::surge::profile))
        .route("/surge/rule-set", get(route::surge::rule_set))
        .route("/clash", get(route::clash::profile))
        .route("/clash/rule-set", get(route::clash::rule_set))
        .route("/subscription_log", get(route::subscription::subscription_logs))
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

    info!("Listening on: http://{}", listen_addr.as_ref());
    warn!("It is recommended to use nginx for reverse proxy and enable SSL");
    info!("usage: all url parameters need to be url-encoded");
    info!(
        "\tmain sub: http://{}/surge?url=[boslife subscription url]",
        listen_addr.as_ref()
    );
    info!(
        "\trule set: http://{}/rule-set?url=[boslife subscription url]&boslife=true",
        listen_addr.as_ref()
    );

    let listener = tokio::net::TcpListener::bind(listen_addr.as_ref()).await?;

    info!("Server starting");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("Server stopped");
    Ok(())
}

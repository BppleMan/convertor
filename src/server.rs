use crate::boslife::boslife_service::BosLifeService;
use crate::server::route::AppState;
use crate::{shutdown_signal, ConvertorCli};
use axum::routing::get;
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tower_http::LatencyUnit;
use tracing::{info, warn};

pub mod route;

pub async fn start_server(
    cli: ConvertorCli,
    service: BosLifeService,
    base_dir: PathBuf,
) -> color_eyre::Result<()> {
    info!("{:#?}", cli);
    info!("base_dir: {:?}", base_dir);

    let app_state = AppState { service };
    let app = Router::new()
        .route("/", get(route::root))
        .route("/surge", get(route::surge::profile))
        .route("/surge/rule-set", get(route::surge::rule_set))
        .route("/clash", get(route::clash::profile))
        .route("/clash/proxy-provider", get(route::clash::proxy_provider))
        .route("/clash/rule-set", get(route::clash::rule_set))
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

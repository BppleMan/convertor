use crate::api::SubProviderWrapper;
use crate::common::config::ConvertorConfig;
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::sub_provider::SubProvider;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_profile::SurgeProfile;
use color_eyre::Result;
use moka::future::Cache;
use std::collections::HashMap;
use std::net::{SocketAddr, SocketAddrV4};
use std::path::Path;
use tokio::signal;
use tracing::{info, warn};
use url::Url;

pub mod clash_router;
pub mod surge_router;
pub mod router;
pub mod error;

pub struct AppState {
    pub config: ConvertorConfig,
    pub api_map: HashMap<SubProvider, SubProviderWrapper>,
    pub surge_cache: Cache<ProfileCacheKey, SurgeProfile>,
    pub clash_cache: Cache<ProfileCacheKey, ClashProfile>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProfileCacheKey {
    pub client: ProxyClient,
    pub provider: SubProvider,
    pub uni_sub_url: Url,
    pub interval: u64,
    pub server: Option<Url>,
    pub strict: Option<bool>,
    pub policy: Option<Policy>,
}

impl AppState {
    pub fn new(config: ConvertorConfig, api_map: HashMap<SubProvider, SubProviderWrapper>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60); // 1 hour
        let surge_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        let clash_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self {
            config,
            api_map,
            surge_cache,
            clash_cache,
        }
    }
}

pub async fn start_server(
    listen_addr: SocketAddrV4,
    config: ConvertorConfig,
    api_map: HashMap<SubProvider, SubProviderWrapper>,
    base_dir: impl AsRef<Path>,
) -> Result<()> {
    info!("base_dir: {}", base_dir.as_ref().display());
    info!("监听中: {}", &listen_addr);
    warn!("建议使用 nginx 等网关进行反向代理，以开启 HTTPS 支持");
    info!("服务启动，使用 Ctrl+C 或 SIGTERM 关闭服务");
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;

    let app_state = AppState::new(config, api_map);
    let app = router::router(app_state);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("服务关闭");
    Ok(())
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

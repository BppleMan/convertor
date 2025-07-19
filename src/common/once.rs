use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_base_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".convertor.dev");
    #[cfg(not(debug_assertions))]
    let base_dir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())).join(".convertor");
    base_dir
}

static INITIALIZED_BACKTRACE: Once = Once::new();

pub fn init_backtrace() {
    INITIALIZED_BACKTRACE.call_once(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
}

static INITIALIZED_LOG: Once = Once::new();

pub fn init_log(base_dir: impl AsRef<Path>) {
    INITIALIZED_LOG.call_once(|| {
        let filter = EnvFilter::new("info")
            .add_directive("convertor=trace".parse().unwrap())
            .add_directive("tower_http=trace".parse().unwrap())
            .add_directive("moka=trace".parse().unwrap());

        let file_appender = tracing_appender::rolling::hourly(base_dir.as_ref().join("logs"), "convertor.log");

        // let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = tracing_subscriber::fmt::layer().with_writer(file_appender);

        let stdout_layer = tracing_subscriber::fmt::layer().pretty();

        tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    });
}

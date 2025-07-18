use std::path::Path;
use std::sync::Once;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod server;

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

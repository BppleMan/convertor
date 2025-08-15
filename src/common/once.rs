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

pub fn init_log() {
    INITIALIZED_LOG.call_once(|| {
        let filter = EnvFilter::new("info")
            .add_directive("convertor=trace".parse().unwrap())
            .add_directive("tower_http=trace".parse().unwrap())
            .add_directive("moka=trace".parse().unwrap());

        // let file_appender = tracing_appender::rolling::hourly(base_dir.as_ref().join("logs"), "convertor.log");
        // let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        // let file_layer = tracing_subscriber::fmt::layer().with_writer(file_appender);

        #[cfg(debug_assertions)]
        let stdout_layer = tracing_subscriber::fmt::layer().pretty();
        #[cfg(not(debug_assertions))]
        let stdout_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(false)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true);

        let registry = tracing_subscriber::registry()
            .with(filter)
            // .with(file_layer)
            .with(stdout_layer);

        if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            use opentelemetry::KeyValue;
            use opentelemetry::global;
            use opentelemetry::trace::TracerProvider;
            use opentelemetry_otlp::SpanExporter;
            use opentelemetry_otlp::WithExportConfig;
            use opentelemetry_sdk::resource::Resource;
            use opentelemetry_sdk::trace::SdkTracerProvider;

            // 0.30：先建 Exporter（gRPC/tonic）
            let exporter = SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .expect("build otlp span exporter");

            // 0.30：Resource 走 builder（new 已私有）
            let resource = Resource::builder()
                .with_service_name("convertor")
                .with_attributes([
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    KeyValue::new(
                        "deployment.environment",
                        std::env::var("ENV").unwrap_or_else(|_| "prod".into()),
                    ),
                ])
                .build();

            // Provider：batch 导出
            let provider = SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(resource)
                .build();

            // （可选）设为全局
            global::set_tracer_provider(provider.clone());

            // 给 tracing-opentelemetry 一个 tracer
            let tracer = provider.tracer("convertor");
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            registry.with(otel_layer).init();
        } else {
            registry.init();
        };
    });
}

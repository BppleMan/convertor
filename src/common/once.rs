use std::path::Path;
use std::sync::Once;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_base_dir() -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir()
        .expect("无法获取当前工作目录")
        .join(".convertor");
    #[cfg(not(debug_assertions))]
    let base_dir = std::path::PathBuf::from(std::env::var("HOME").expect("没有找到 HOME 目录")).join(".convertor");
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

pub fn init_log(base_dir: Option<&Path>) {
    INITIALIZED_LOG.call_once(|| {
        let logs_dir = base_dir.map(|b| b.join("logs"));
        #[cfg(debug_assertions)]
        println!("Initializing log for {:?}", logs_dir);

        // 1. 灵活 EnvFilter（支持 RUST_LOG，否则用默认）
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
            .add_directive("convertor=trace".parse().unwrap())
            .add_directive("tower_http=trace".parse().unwrap())
            .add_directive("moka=trace".parse().unwrap());

        // 2. 文件日志（每小时滚动）
        let file_layer = logs_dir.map(|logs_dir| {
            let file_appender = tracing_appender::rolling::hourly(logs_dir, "convertor.log");
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(false)
        });

        // 3. 控制台日志（开发模式用 pretty，生产可换 compact）
        let stdout_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
            .with_target(true);

        let registry = tracing_subscriber::registry()
            .with(filter)
            .with(file_layer)
            .with(stdout_layer);

        #[cfg(feature = "otel")]
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
        }

        #[cfg(not(feature = "otel"))]
        registry.init();
    });
}

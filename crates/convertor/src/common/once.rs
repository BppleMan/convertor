use crate::env::Env;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use std::io::IsTerminal;
use std::sync::Once;
use tracing_loki::BackgroundTask;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub const HOME_CONFIG_DIR: &str = ".convertor";
pub const K8S_CONFIG_DIR: &str = "/etc/convertor";

pub fn init_base_dir() -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    let base_dir = std::env::current_dir().expect("无法获取当前工作目录").join(HOME_CONFIG_DIR);
    #[cfg(not(debug_assertions))]
    let base_dir = std::path::PathBuf::from(std::env::var("HOME").expect("没有找到 HOME 目录")).join(HOME_CONFIG_DIR);
    base_dir
}

static INITIALIZED_BACKTRACE: Once = Once::new();

pub fn init_backtrace<F>(call_once: F)
where
    F: FnOnce(),
{
    // INITIALIZED_BACKTRACE.call_once(|| {
    //     if let Err(e) = color_eyre::install() {
    //         eprintln!("Failed to install color_eyre: {e}");
    //     }
    // });
    INITIALIZED_BACKTRACE.call_once(call_once);
}

static INITIALIZED_LOG: Once = Once::new();
// pub static LOKI_TASK: OnceLock<Arc<Pin<BackgroundTask>>> = OnceLock::new();

macro_rules! layer {
    (env_filter) => {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
            .add_directive("convertor=trace".parse().unwrap())
            .add_directive("convd=trace".parse().unwrap())
            .add_directive("confly=trace".parse().unwrap())
            .add_directive("tower_http=trace".parse().unwrap())
            .add_directive("moka=trace".parse().unwrap())
            .add_directive("reqwest=trace".parse().unwrap())
            .add_directive("httpv=trace".parse().unwrap())
    };
    (fmt_layer) => {
        tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_ansi(std::io::stdout().is_terminal())
            .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
            .pretty()
    };
    (loki_layer, $loki_url:expr, $service:expr) => {
        tracing_loki::builder()
            .label("service", $service)
            .expect("无法设置 loki label: service")
            .label("env", Env::current())
            .expect("无法设置 loki label: env")
            .extra_field("service", $service)
            .expect("无法设置 loki extra_field: service")
            .extra_field("env", Env::current())
            .expect("无法设置 loki extra_field: env")
            .build_url($loki_url.parse().expect("loki url"))
            .expect("无法创建 loki 层")
    };
    (otlp_layer, $otlp_grpc:expr, $service:expr) => {
        tracing_opentelemetry::layer().with_tracer(
            opentelemetry_sdk::trace::SdkTracerProvider::builder()
                .with_batch_exporter(
                    opentelemetry_otlp::SpanExporter::builder()
                        .with_tonic()
                        .with_endpoint($otlp_grpc)
                        .with_timeout(std::time::Duration::from_secs(2))
                        .build()
                        .expect("failed to create otlp exporter"),
                )
                .with_resource(
                    opentelemetry_sdk::Resource::builder()
                        .with_service_name($service)
                        .with_attribute(opentelemetry::KeyValue::new("environment", Env::current().name()))
                        .build(),
                )
                .build()
                .tracer("convd"),
        )
    };
}

pub fn init_log(loki_url: Option<&str>, otlp_grpc: Option<&str>) -> Option<BackgroundTask> {
    let mut loki_task_guard = None;
    INITIALIZED_LOG.call_once(|| {
        // 1. 灵活 EnvFilter（支持 RUST_LOG，否则用默认）
        let filter = layer!(env_filter);

        // 2. 控制台日志（开发模式用 pretty，生产可换 compact）
        let fmt_layer = layer!(fmt_layer);

        // 3. loki 日志（可选）
        let service = "convd";
        let loki_layer = loki_url.map(|loki_url| {
            let (loki_layer, loki_task) = layer!(loki_layer, loki_url, service);
            loki_task_guard.replace(loki_task);
            loki_layer
        });

        // 4. otel 日志（可选）
        match (loki_layer, otlp_grpc) {
            (Some(loki_layer), Some(otlp_grpc)) => {
                let otlp_layer = layer!(otlp_layer, otlp_grpc, service);
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .with(loki_layer)
                    .with(otlp_layer)
                    .init();
            }
            (Some(loki_layer), None) => {
                eprintln!("无法读取 OTLP_GRPC, 不启用 otlp 日志");
                tracing_subscriber::registry().with(filter).with(fmt_layer).with(loki_layer).init();
            }
            (None, Some(otlp_grpc)) => {
                eprintln!("无法读取 LOKI_URL, 不启用 loki 日志");
                let otlp_layer = layer!(otlp_layer, otlp_grpc, service);
                tracing_subscriber::registry().with(filter).with(fmt_layer).with(otlp_layer).init();
            }
            (None, None) => {
                eprintln!("无法读取 LOKI_URL, 不启用 loki 日志");
                eprintln!("无法读取 OTLP_GRPC, 不启用 otlp 日志");
                tracing_subscriber::registry()
                    .with(layer!(env_filter))
                    .with(layer!(fmt_layer))
                    .init();
            }
        }
    });
    loki_task_guard
}

use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let (layer, task) = tracing_loki::builder()
        .label("service", "test-loki")
        .unwrap()
        .build_url("http://localhost:3100".parse().unwrap())
        .unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(layer)
        .init();

    tokio::spawn(task);

    info!("测试日志 1");
    info!("测试日志 2");
    info!("测试日志 3");

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    println!("测试完成，等待 3 秒后退出");
}

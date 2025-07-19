use convertor::common::config::ConvertorConfig;
use convertor::common::proxy_client::ProxyClient;
use convertor::core::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::init_backtrace;
use std::path::Path;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = std::env::current_dir()?.join("convertor-core").join(".convertor.bench");
    init_backtrace();
    // 下面两种方案任选一
    #[cfg(feature = "bench")]
    tracing_span_tree::span_tree().aggregate(true).enable();
    // #[cfg(feature = "bench")]
    // tracing_profile::init_tracing()?;

    let config = ConvertorConfig::search(&base_dir, Option::<&Path>::None)?;
    let url = config.create_convertor_url(ProxyClient::Surge)?;

    let file = std::fs::read_to_string(base_dir.join("mock.conf"))?;
    let mut profile = SurgeProfile::parse(file)?;
    profile.optimize(&url)?;

    Ok(())
}

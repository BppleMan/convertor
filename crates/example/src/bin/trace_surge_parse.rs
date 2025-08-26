use convertor::common::once::init_backtrace;
use convertor::config::ConvertorConfig;
use convertor::config::provider_config::Provider;
use convertor::config::proxy_client_config::ProxyClient;
use convertor::core::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use std::path::Path;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".convertor.bench");
    init_backtrace();
    // 下面两种方案任选一
    #[cfg(feature = "bench")]
    tracing_span_tree::span_tree().aggregate(true).enable();
    // #[cfg(feature = "bench")]
    // tracing_profile::init_tracing()?;

    let config = ConvertorConfig::template();
    let url_builder = config.create_url_builder(ProxyClient::Surge, Provider::BosLife)?;

    let file = std::fs::read_to_string(base_dir.join("mock.conf"))?;
    let mut profile = SurgeProfile::parse(file)?;
    profile.convert(&url_builder)?;

    Ok(())
}

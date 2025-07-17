use convertor::convertor_config::ConvertorConfig;
use convertor::core::profile::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::init_backtrace;
use std::path::Path;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let base_dir = std::env::current_dir()?.join(".convertor.bench");
    init_backtrace();
    // 下面两种方案任选一
    #[cfg(feature = "bench")]
    tracing_span_tree::span_tree().aggregate(true).enable();
    // #[cfg(feature = "bench")]
    // tracing_profile::init_tracing()?;

    let config = ConvertorConfig::search(&base_dir, Option::<&Path>::None)?;
    let url_builder = config.create_url_builder()?;

    let file = std::fs::read_to_string(base_dir.join("mock.conf"))?;
    let mut profile = SurgeProfile::parse(file)?;
    profile.optimize(&url_builder)?;

    Ok(())
}

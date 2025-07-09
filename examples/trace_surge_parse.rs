use convertor::init_backtrace;
use convertor::profile::core::clash_profile::ClashProfile;
use convertor::profile::core::profile::Profile;
use convertor::profile::core::surge_profile::SurgeProfile;

fn main() -> color_eyre::Result<()> {
    let base_dir = std::env::current_dir()?.join(".convertor.bench");
    init_backtrace();
    // 下面两种方案任选一
    #[cfg(feature = "bench")]
    tracing_span_tree::span_tree().aggregate(true).enable();
    // #[cfg(feature = "bench")]
    // tracing_profile::init_tracing()?;

    let file = std::fs::read_to_string(base_dir.join("mock.yaml"))?;
    ClashProfile::parse(file)?;

    let file = std::fs::read_to_string(base_dir.join("mock.conf"))?;
    SurgeProfile::parse(file)?;

    Ok(())
}

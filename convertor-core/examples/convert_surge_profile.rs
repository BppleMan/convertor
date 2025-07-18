use convertor_core::api::ServiceApi;
use convertor_core::client::Client;
use convertor_core::config::ConvertorConfig;
use convertor_core::core::profile::Profile;
use convertor_core::core::profile::surge_profile::SurgeProfile;
use convertor_core::core::renderer::Renderer;
use convertor_core::core::renderer::surge_renderer::SurgeRenderer;
use convertor_core::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir);

    let raw_sub_content = api.get_raw_profile(Client::Surge).await?;
    let mut profile = SurgeProfile::parse(raw_sub_content)?;
    let url = config.create_convertor_url(Client::Surge)?;
    profile.optimize(&url)?;

    let converted = SurgeRenderer::render_profile(&profile)?;
    println!("{converted}");

    Ok(())
}

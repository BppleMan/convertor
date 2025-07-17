use convertor::client::Client;
use convertor::convertor_config::ConvertorConfig;
use convertor::core::profile::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use convertor::service_provider::api::ServiceApi;
use convertor::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let client = reqwest::Client::new();
    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir, client);

    let raw_sub_content = api.get_raw_profile(Client::Surge).await?;
    let mut profile = SurgeProfile::parse(raw_sub_content)?;
    let url_builder = config.create_url_builder()?;
    profile.optimize(&url_builder)?;

    let converted = SurgeRenderer::render_profile(&profile)?;
    println!("{converted}");

    Ok(())
}

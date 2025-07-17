use convertor::client::Client;
use convertor::config::convertor_config::ConvertorConfig;
use convertor::core::profile::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use convertor::service_provider::subscription_api::boslife_api::BosLifeApi;
use convertor::{init_backtrace, init_base_dir};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    let convertor_config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    let client = reqwest::Client::new();
    let api = BosLifeApi::new(&base_dir, client, convertor_config.service_config.clone());

    let raw_sub_url = api
        .get_raw_sub_url(convertor_config.service_config.base_url.clone(), Client::Surge)
        .await?;
    let url_builder = convertor_config.create_url_builder(raw_sub_url)?;

    // raw_sub_url 是通用订阅地址, sub_url 是指定客户端的订阅地址
    let sub_url = url_builder.build_subscription_url(Client::Surge)?;
    let raw_sub_content = api.get_raw_profile(sub_url, Client::Surge).await?;
    let mut profile = SurgeProfile::parse(raw_sub_content)?;
    profile.optimize(&url_builder)?;

    let converted = SurgeRenderer::render_profile(&profile)?;
    println!("{converted}");

    Ok(())
}

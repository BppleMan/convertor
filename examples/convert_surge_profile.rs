use convertor_core::api::ServiceApi;
use convertor_core::common::config::ConvertorConfig;
use convertor_core::common::once::{init_backtrace, init_base_dir};
use convertor_core::common::proxy_client::ProxyClient;
use convertor_core::core::profile::Profile;
use convertor_core::core::profile::surge_profile::SurgeProfile;
use convertor_core::core::renderer::Renderer;
use convertor_core::core::renderer::surge_renderer::SurgeRenderer;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    // 搜索可用配置文件
    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    // 创建服务(BosLife)API实例
    let api = ServiceApi::get_service_provider_api(config.service_config.clone(), &base_dir);
    // 创建 URL 对象，该 URL 用于从convertor订阅优化后的配置文件
    let url = config.create_convertor_url(ProxyClient::Surge)?;

    // 获取原始的 Surge 配置文件内容
    let raw_sub_content = api.get_raw_profile(ProxyClient::Surge).await?;
    // 解析原始配置文件内容为 SurgeProfile 对象
    let mut profile = SurgeProfile::parse(raw_sub_content)?;
    // 转换 SurgeProfile 对象，convertor 的核心作用
    profile.convert(&url)?;

    // 使用渲染器将 SurgeProfile 对象转换为字符串格式
    let converted = SurgeRenderer::render_profile(&profile)?;
    println!("{converted}");

    Ok(())
}

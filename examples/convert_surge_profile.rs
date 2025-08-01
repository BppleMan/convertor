use color_eyre::eyre::eyre;
use convertor::api::SubProviderWrapper;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::config::sub_provider::SubProvider;
use convertor::common::once::{init_backtrace, init_base_dir};
use convertor::core::profile::Profile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = init_base_dir();
    init_backtrace();

    // 确定适用的客户端和订阅提供者
    // 这里使用 Surge 客户端和 BosLife 机场
    let client = ProxyClient::Surge;
    let provider = SubProvider::BosLife;

    // 搜索可用配置文件
    let config = ConvertorConfig::search(&base_dir, Option::<&str>::None)?;
    // 创建订阅供应商实例
    let api_map = SubProviderWrapper::create_api(config.providers.clone(), &base_dir);
    // 获取 BosLife 的 API 实例
    let api = api_map
        .get(&provider)
        .ok_or_else(|| eyre!("未找到 BosLife 订阅提供者"))?;

    // 获取原始订阅配置文件内容: 来源于 BosLife 机场;适用于 Surge
    let raw_sub_content = api.get_raw_profile(client).await?;
    // 解析原始配置文件内容为 SurgeProfile 对象
    let mut profile = SurgeProfile::parse(raw_sub_content)?;
    // 创建 UrlBuilder 对象, 该 UrlBuilder 可用于创建适用于 Surge 的且使用 BosLife 订阅的 URL
    let url_builder = config.create_url_builder(ProxyClient::Surge, SubProvider::BosLife)?;
    // 转换 SurgeProfile 对象
    // 传入 UrlBuilder 对象有两个作用
    // - 用于生成 Surge 配置的托管链接
    // - 用于生成 Surge 规则集的托管链接
    // 二者均会指向 convertor 所在服务器
    profile.convert(&url_builder)?;

    // 使用渲染器将 SurgeProfile 对象转换为字符串格式
    let converted = SurgeRenderer::render_profile(&profile)?;
    println!("{converted}");

    Ok(())
}

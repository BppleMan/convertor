use crate::init_test;
use crate::server::mock::start_mock_provider_server;
use color_eyre::Report;
use convertor::api::UniversalProviderApi;
use convertor::cli::sub_provider_executor::{SubProviderCmd, SubProviderExecutor};
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client::ProxyClient;
use std::path::Path;

use pretty_assertions::assert_str_eq;

trait TestConfig {
    fn test(base_dir: impl AsRef<Path>) -> Result<ConvertorConfig, Report>;
}

impl TestConfig for ConvertorConfig {
    fn test(base_dir: impl AsRef<Path>) -> Result<ConvertorConfig, Report> {
        let base_dir = base_dir.as_ref();
        let mut convertor_config = ConvertorConfig::template()?;
        if let Some(surge_config) = convertor_config.client.surge.as_mut() {
            surge_config.set_surge_dir(base_dir.join("provider/surge").display().to_string());
        };
        if let Some(clash_config) = convertor_config.client.clash.as_mut() {
            clash_config.set_clash_dir(base_dir.join("provider/clash").display().to_string());
        };
        Ok(convertor_config)
    }
}

/// convertor sub surge
#[tokio::test]
async fn test_sub_surge() -> Result<(), Report> {
    let base_dir = init_test();
    let mut convertor_config = ConvertorConfig::test(&base_dir)?;

    // 启动 mock provider server，自动拦截并返回 raw_profile
    let mock_server = start_mock_provider_server(&convertor_config.provider).await?;
    let sub_provider_base_url = mock_server.base_url();
    println!("Mock provider server started at: {sub_provider_base_url}");
    convertor_config
        .provider
        .uni_sub_url
        .set_port(Some(mock_server.port()))
        .map_err(|_| Report::msg("can't set mock server port"))?;
    convertor_config.provider.api_host = mock_server.base_url().parse()?;

    let api = UniversalProviderApi::get_service_provider_api(convertor_config.provider.clone(), &base_dir);
    let executor = SubProviderExecutor::new(convertor_config, api);

    let cmd = SubProviderCmd {
        client: ProxyClient::Surge,
        ..Default::default()
    };
    let result = executor.execute(cmd).await?;
    println!("{result}");

    // 构造期望值
    let expect_raw_url = format!("{sub_provider_base_url}/subscription?token=bppleman&flag=surge");
    let expect_convertor_url = "http://127.0.0.1:8001/profile?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&uni_sub_url=";
    let expect_logs_url = "http://127.0.0.1:8001/sub-logs?secret=yIZRFPp9wbO0w4Zp:szY+7bAJ2ilkb4onQdsT94op3OJ/pn8I&page_current=1&page_size=20";
    let expect_policy_labels = vec![
        "规则集: [BosLife] by convertor/2.4.5",
        "规则集: [BosLife: force-remote-dns] by convertor/2.4.5",
        "规则集: [BosLife: no-resolve] by convertor/2.4.5",
        "规则集: [Subscription] by convertor/2.4.5",
    ];
    let expect_policy_urls = vec![
        "http://127.0.0.1:8001/rule-provider?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&policy.name=BosLife&policy.is_subscription=false&uni_sub_url=QTjWEKwENPcs+hr7:XlcT9Sb3zQf0uuu5tWRl3n7pMOcZMhRO8l6XsjpESpvUUr8XwwqdCSH9S6Q+hQ5PpiomuYX32nXCgynm9rCKOWvf",
        "http://127.0.0.1:8001/rule-provider?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&policy.name=BosLife&policy.option=force-remote-dns&policy.is_subscription=false&uni_sub_url=QTjWEKwENPcs+hr7:XlcT9Sb3zQf0uuu5tWRl3n7pMOcZMhRO8l6XsjpESpvUUr8XwwqdCSH9S6Q+hQ5PpiomuYX32nXCgynm9rCKOWvf",
        "http://127.0.0.1:8001/rule-provider?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&policy.name=BosLife&policy.option=no-resolve&policy.is_subscription=false&uni_sub_url=QTjWEKwENPcs+hr7:XlcT9Sb3zQf0uuu5tWRl3n7pMOcZMhRO8l6XsjpESpvUUr8XwwqdCSH9S6Q+hQ5PpiomuYX32nXCgynm9rCKOWvf",
        "http://127.0.0.1:8001/rule-provider?client=surge&server=http://127.0.0.1:8001/&interval=86400&strict=true&policy.name=DIRECT&policy.is_subscription=true&uni_sub_url=QTjWEKwENPcs+hr7:XlcT9Sb3zQf0uuu5tWRl3n7pMOcZMhRO8l6XsjpESpvUUr8XwwqdCSH9S6Q+hQ5PpiomuYX32nXCgynm9rCKOWvf",
    ];

    // 完全匹配断言
    assert_eq!(result.raw_link.label, "原始订阅链接:");
    assert_eq!(result.raw_link.url.as_str(), expect_raw_url);
    assert_eq!(result.convertor_link.label, "转换器订阅链接:");
    assert_eq!(result.convertor_link.url.as_str(), expect_convertor_url);
    assert_eq!(result.logs_link.label, "订阅日志链接:");
    assert_eq!(result.logs_link.url.as_str(), expect_logs_url);
    assert_eq!(result.policy_links.len(), expect_policy_labels.len());
    for (i, link) in result.policy_links.iter().enumerate() {
        assert_eq!(link.label, expect_policy_labels[i]);
        assert_eq!(link.url.as_str(), expect_policy_urls[i]);
    }

    Ok(())
}

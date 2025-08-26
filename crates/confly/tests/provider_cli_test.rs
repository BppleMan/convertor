use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client_config::ProxyClient;
use convertor::provider_api::ProviderApi;

use confly::cli::provider_cli::{ProviderCli, ProviderCmd};
use convertor::common::config::provider_config::Provider;
use convertor::testkit::{init_test, start_mock_provider_server};
use url::Url;

pub fn cmds(client: ProxyClient, provider: Provider) -> [ProviderCmd; 3] {
    [
        ProviderCmd {
            client,
            provider,
            ..Default::default()
        },
        ProviderCmd {
            client,
            provider,
            server: Some(Url::parse("http://localhost:8080").expect("不合法的服务器地址")),
            ..Default::default()
        },
        ProviderCmd {
            client,
            provider,
            interval: Some(86400),
            strict: Some(false),
            ..Default::default()
        },
    ]
}

#[tokio::test]
async fn test_subscription_surge_boslife() -> color_eyre::Result<()> {
    let client = ProxyClient::Surge;
    let provider = Provider::BosLife;
    init_test();
    let mut config = ConvertorConfig::template();
    config.providers.values_mut().for_each(|provider| {
        provider.api_config.headers.remove("Authorization");
    });
    start_mock_provider_server(&mut config).await?;
    let cmds = cmds(client, provider);
    for (i, cmd) in cmds.into_iter().enumerate() {
        let ctx = format!("test_subscription_surge_boslife_cmd_{i}");
        let api_map = ProviderApi::create_api_no_redis(config.providers.clone());
        let mut executor = ProviderCli::new(config.clone(), api_map);
        let (url_builder, result) = executor.execute(cmd).await?;
        let result = result.to_string();
        let result = result
            .replace(
                &url_builder
                    .sub_url
                    .port()
                    .map(|p| p.to_string())
                    .unwrap_or("".to_string()),
                "<PORT>",
            )
            .replace(&url_builder.server.to_string(), "<SERVER>")
            .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>");
        insta::assert_snapshot!(ctx, result);
    }
    Ok(())
}

#[tokio::test]
async fn test_subscription_clash_boslife() -> color_eyre::Result<()> {
    init_test();
    let client = ProxyClient::Clash;
    let provider = Provider::BosLife;
    let mut config = ConvertorConfig::template();
    config.providers.values_mut().for_each(|provider| {
        provider.api_config.headers.remove("Authorization");
    });
    start_mock_provider_server(&mut config).await?;
    let cmds = cmds(client, provider);
    for (i, cmd) in cmds.into_iter().enumerate() {
        let ctx = format!("test_subscription_clash_boslife_cmd_{i}");
        let api_map = ProviderApi::create_api_no_redis(config.providers.clone());
        let mut executor = ProviderCli::new(config.clone(), api_map);
        let (url_builder, result) = executor.execute(cmd).await?;
        let result = result.to_string();
        let result = result
            .replace(
                &url_builder
                    .sub_url
                    .port()
                    .map(|p| p.to_string())
                    .unwrap_or("".to_string()),
                "<PORT>",
            )
            .replace(&url_builder.server.to_string(), "<SERVER>")
            .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>");
        insta::assert_snapshot!(ctx, result);
    }
    Ok(())
}

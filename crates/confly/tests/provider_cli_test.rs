use color_eyre::Report;
use color_eyre::eyre::eyre;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client_config::ProxyClient;
use convertor::provider_api::ProviderApi;

use confly::cli::provider_cli::{ProviderCli, ProviderCmd};
use convertor::common::config::provider_config::Provider;
use convertor::testkit::{init_test, start_mock_provider_server};
use convertor::url::url_builder::HostPort;
use pretty_assertions::assert_str_eq;
use regex::Regex;
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
            interval: Some(43200),
            strict: Some(false),
            ..Default::default()
        },
    ]
}

async fn test_subscription(
    convertor_config: &ConvertorConfig,
    client: ProxyClient,
    provider: Provider,
    cmd: ProviderCmd,
) -> Result<(), Report> {
    let convertor_config = convertor_config.clone();
    let server = match &cmd.server {
        Some(server) => server.clone(),
        None => convertor_config.server.clone(),
    };
    let client_config = convertor_config
        .clients
        .get(&client)
        .ok_or_else(|| eyre!("未找到客户端配置: {}", client))?;
    let interval = cmd.interval.unwrap_or(client_config.interval());
    let strict = cmd.strict.unwrap_or(client_config.strict());

    let api_map = ProviderApi::create_api_no_redis(convertor_config.providers.clone());
    let mut executor = ProviderCli::new(convertor_config, api_map);
    let (url_builder, result) = executor.execute(cmd).await?;

    // 构造期望值
    let expect_raw_url = format!(
        "{}://{}/subscription?token=bppleman&flag={}",
        url_builder.sub_url.scheme(),
        url_builder.sub_url.host_port()?,
        client
    );
    assert_str_eq!(expect_raw_url, result.raw_url.to_string());

    let expect_profile_url = format!(
        "{server}profile?client={client}&provider={provider}&server={server}&interval={interval}&strict={strict}&sub_url={}",
        url_builder.enc_sub_url
    );
    assert_str_eq!(expect_profile_url, result.profile_url.to_string());

    let regex_str = format!(
        r#"{}sub-logs\?provider={provider}&secret=(?P<enc_secret>.+)&page=1&page_size=20"#,
        server.as_str().replace(".", "\\."),
    );
    let regex = Regex::new(&regex_str)?;
    let sub_logs_url_string = result.sub_logs_url.to_string();
    let Some(captures) = regex.captures(&sub_logs_url_string) else {
        panic!("未正确捕获订阅日志链接");
    };
    let Some(enc_secret) = captures.name("enc_secret") else {
        panic!("未正确捕获 enc_secret");
    };
    executor.config.validate_enc_secret(enc_secret.as_str())?;

    let expect_policy_urls = [
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=DIRECT&policy.is_subscription=true&sub_url={}",
            url_builder.enc_sub_url
        ),
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=BosLife&policy.is_subscription=false&sub_url={}",
            url_builder.enc_sub_url
        ),
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=BosLife&policy.option=no-resolve&policy.is_subscription=false&sub_url={}",
            url_builder.enc_sub_url
        ),
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=BosLife&policy.option=force-remote-dns&policy.is_subscription=false&sub_url={}",
            url_builder.enc_sub_url
        ),
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=DIRECT&policy.is_subscription=false&sub_url={}",
            url_builder.enc_sub_url
        ),
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=DIRECT&policy.option=no-resolve&policy.is_subscription=false&sub_url={}",
            url_builder.enc_sub_url
        ),
        format!(
            "{server}rule-provider?client={client}&provider={provider}&server={server}&interval={interval}&policy.name=DIRECT&policy.option=force-remote-dns&policy.is_subscription=false&sub_url={}",
            url_builder.enc_sub_url
        ),
    ];

    for (i, url) in result.rule_provider_urls.iter().enumerate() {
        assert_str_eq!(expect_policy_urls[i], url.to_string());
    }

    Ok(())
}

#[tokio::test]
async fn test_subscription_surge_boslife() -> color_eyre::Result<()> {
    init_test();
    let mut config = ConvertorConfig::template();
    config.providers.values_mut().for_each(|provider| {
        provider.api_config.headers.remove("Authorization");
    });
    // {
    //     调试
    //     let api_map = ProviderApi::create_api_no_redis(config.providers.clone());
    //     let provider_api = api_map
    //         .get(&Provider::BosLife)
    //         .ok_or_else(|| eyre!("未找到 BosLife 提供者"))?;
    //     provider_api.login().await?;
    // }
    start_mock_provider_server(&mut config).await?;
    let client = ProxyClient::Surge;
    let provider = Provider::BosLife;
    let cmds = cmds(client, provider);
    for cmd in cmds {
        test_subscription(&config, client, provider, cmd).await?;
    }
    Ok(())
}

// #[tokio::test]
// async fn test_subscription_clash_boslife() -> color_eyre::Result<()> {
//     let convertor_config = convertor_config().await;
//     let client = ProxyClient::Clash;
//     let provider = Provider::BosLife;
//     let cmds = cmds(client, provider);
//     for cmd in cmds {
//         test_subscription(&convertor_config, client, provider, cmd).await?;
//     }
//     Ok(())
// }

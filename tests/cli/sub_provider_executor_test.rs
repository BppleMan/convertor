use color_eyre::Report;
use color_eyre::eyre::eyre;
use convertor::api::SubProviderWrapper;
use convertor::cli::sub_provider_executor::{SubProviderCmd, SubProviderExecutor};
use convertor::common::config::ConvertorConfig;
use convertor::common::config::proxy_client::ProxyClient;

use crate::server::{redis_url, start_mock_provider_server};
use convertor::common::config::sub_provider::SubProvider;
use convertor::common::redis_info::{init_redis_info, redis_client};
use convertor::core::url_builder::HostPort;
use pretty_assertions::assert_str_eq;
use regex::Regex;
use rstest::{fixture, rstest};
use url::Url;

#[fixture]
#[once]
pub fn convertor_config() -> ConvertorConfig {
    let mut config = ConvertorConfig::template();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async { start_mock_provider_server(&mut config.providers).await.unwrap() });
        config
    })
    .join()
    .unwrap()
}

#[fixture]
pub async fn connection_manager() -> redis::aio::ConnectionManager {
    init_redis_info().expect("Failed to init redis client");
    let redis = redis_client(redis_url()).expect("无法连接到 Redis");
    redis::aio::ConnectionManager::new(redis)
        .await
        .expect("无法创建 Redis 连接管理器")
}

#[rstest]
#[tokio::test]
async fn test_subscription(
    convertor_config: &ConvertorConfig,
    connection_manager: impl std::future::Future<Output = redis::aio::ConnectionManager>,
    #[values(ProxyClient::Surge, ProxyClient::Clash)] client: ProxyClient,
    #[values(SubProvider::BosLife)] provider: SubProvider,
    #[values(
        SubProviderCmd {
            client,
            provider,
            ..Default::default()
        },
        SubProviderCmd {
            client,
            provider,
            server: Some(Url::parse("http://localhost:8080").expect("不合法的服务器地址")),
            ..Default::default()
        },
        SubProviderCmd {
            client,
            provider,
            interval: Some(43200),
            strict: Some(false),
            ..Default::default()
        },
    )]
    cmd: SubProviderCmd,
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

    let api_map = SubProviderWrapper::create_api(convertor_config.providers.clone(), connection_manager.await);
    let mut executor = SubProviderExecutor::new(convertor_config, api_map);
    let (url_builder, result) = executor.execute(cmd).await?;

    // 构造期望值
    let expect_raw_url = format!(
        "{}://{}/subscription?token=bppleman&flag={}",
        url_builder.sub_url.scheme(),
        url_builder.sub_url.host_port()?,
        client
    );
    assert_str_eq!(expect_raw_url, result.raw_link.url.as_str());

    let expect_profile_url = format!(
        "{server}profile?client={client}&provider={provider}&server={server}&interval={interval}&strict={strict}&sub_url={}",
        url_builder.enc_sub_url
    );
    assert_str_eq!(expect_profile_url, result.profile_link.url.as_str());

    let regex_str = format!(
        r#"{}sub-logs\?provider={provider}&secret=(?P<enc_secret>.+)&page=1&page_size=20"#,
        server.as_str().replace(".", "\\."),
    );
    let regex = Regex::new(&regex_str)?;
    let Some(captures) = regex.captures(result.logs_link.url.as_str()) else {
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

    for (i, link) in result.rule_provider_links.iter().enumerate() {
        assert_str_eq!(expect_policy_urls[i], link.url.as_str());
    }

    Ok(())
}

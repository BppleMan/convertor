use crate::common::config::ConvertorConfig;
use crate::common::config::provider_config::ProviderConfig;
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::encrypt::nonce_rng_use_seed;
use crate::common::once::{init_backtrace, init_log};
use crate::core::profile::policy::Policy;
use crate::url::url_builder::HostPort;
use color_eyre::Report;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use std::path::{Path, PathBuf};
use strum::VariantArray;
use url::Url;

pub fn init_test() -> PathBuf {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-assets");
    init_backtrace();
    init_log(None);
    nonce_rng_use_seed([0u8; 32]);
    base_dir
}

pub async fn start_mock_provider_server(config: &mut ConvertorConfig) -> Result<(), Report> {
    for (_, config) in config.providers.iter_mut() {
        config.start_mock_provider_server().await?;
    }
    Ok(())
}

pub(crate) trait MockServerExt {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report>;
}

impl MockServerExt for ProviderConfig {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report> {
        let mock_server = MockServer::start_async().await;

        // 将订阅地址导航至 mock server 的 /subscription 路径
        let subscribe_url_path = "/subscription";
        let token = "bppleman";

        self.sub_url =
            Url::parse(&mock_server.url(format!("{subscribe_url_path}?token={token}"))).expect("不合法的订阅地址");
        self.api_config.host = Url::parse(&mock_server.url("/")).expect("不合法的 API 地址");

        mock_server
            .mock_async(|when, then| {
                when.method(POST).path(self.api_config.login_path());
                let body = serde_json::json!({
                    "data": {
                        "auth_data": "mock_auth_token"
                    }
                });
                then.status(200)
                    .body(serde_json::to_string(&body).unwrap())
                    .header("Content-Type", "application/json");
            })
            .await;

        mock_server
            .mock_async(|when, then| {
                when.method(GET).path(self.api_config.get_sub_path());
                let body = serde_json::json!({
                    "data": {
                        "subscribe_url": mock_server.url(format!("{subscribe_url_path}?token={token}")),
                    }
                });
                then.status(200)
                    .body(serde_json::to_string(&body).unwrap())
                    .header("Content-Type", "application/json");
            })
            .await;

        mock_server
            .mock_async(|when, then| {
                when.method(POST).path(self.api_config.reset_sub_path());
                let body = serde_json::json!({
                    "data": mock_server.url(format!("{subscribe_url_path}?token=reset_{token}")),
                });
                then.status(200)
                    .body(serde_json::to_string(&body).unwrap())
                    .header("Content-Type", "application/json");
            })
            .await;

        // hook mock server 的 /subscription 路径，返回相应的 mock 数据
        let sub_host = self.sub_url.host_port()?;
        for client in ProxyClient::VARIANTS {
            mock_server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path(subscribe_url_path)
                        .query_param("flag", client.as_ref())
                        .query_param("token", token);
                    let body = mock_profile(*client, &sub_host);
                    then.status(200)
                        .body(body)
                        .header("Content-Type", "text/plain; charset=utf-8");
                })
                .await;
        }

        Ok(mock_server)
    }
}

pub fn mock_profile(client: ProxyClient, sub_host: impl AsRef<str>) -> String {
    match client {
        ProxyClient::Surge => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-assets/surge/mock_profile.conf"
        )),
        ProxyClient::Clash => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test-assets/clash/mock_profile.yaml"
        )),
    }
    .replace("{sub_host}", sub_host.as_ref())
}

pub fn policies() -> [Policy; 7] {
    [
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::direct_policy(None),
        Policy::direct_policy(Some("no-resolve")),
        Policy::direct_policy(Some("force-remote-dns")),
    ]
}

use axum::Router;
use axum::routing::get;
use color_eyre::Report;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::provider::{BosLifeConfig, SubProvider, SubProviderConfig};
use convertor::common::config::proxy_client::ProxyClient;
use convertor::common::redis_info::{REDIS_CONVERTOR_PASSWORD, REDIS_CONVERTOR_USERNAME, REDIS_ENDPOINT};
use convertor::core::url_builder::HostPort;
use convertor::provider_api::ProviderApi;
use convertor::server::app_state::AppState;
use convertor::server::router::{profile, raw_profile, rule_provider};
use dispatch_map::DispatchMap;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use std::sync::Arc;
use strum::VariantArray;
use url::Url;

mod profile_test;
mod rule_provider_test;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

pub fn redis_url() -> String {
    format!(
        "rediss://{}:{}@{}/2?protocol=resp3",
        REDIS_CONVERTOR_USERNAME
            .get()
            .expect("REDIS_CONVERTOR_USERNAME not set"),
        REDIS_CONVERTOR_PASSWORD
            .get()
            .expect("REDIS_CONVERTOR_PASSWORD not set"),
        REDIS_ENDPOINT.get().expect("REDIS_ENDPOINT not set")
    )
}

pub async fn start_server() -> color_eyre::Result<ServerContext> {
    let mut config = ConvertorConfig::template();
    start_mock_provider_server(&mut config.providers).await?;

    let api = ProviderApi::create_api_no_redis(config.providers.clone());
    let app_state = Arc::new(AppState::new(config, api));
    let app: Router = Router::new()
        .route("/raw-profile/{client}/{provider}", get(raw_profile))
        .route("/profile/{client}/{provider}", get(profile))
        .route("/rule-provider/{client}/{provider}", get(rule_provider))
        .with_state(app_state.clone());

    Ok(ServerContext { app, app_state })
}

pub async fn start_mock_provider_server(
    providers: &mut DispatchMap<SubProvider, SubProviderConfig>,
) -> Result<(), Report> {
    for (_, config) in providers.iter_mut() {
        match config {
            SubProviderConfig::BosLife(config) => config.start_mock_provider_server().await?,
        };
    }
    Ok(())
}

pub(crate) trait MockServerExt {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report>;
}

impl MockServerExt for BosLifeConfig {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report> {
        let mock_server = MockServer::start_async().await;

        // 将订阅地址导航至 mock server 的 /subscription 路径
        let subscribe_url_path = "/subscription";
        let token = "bppleman";

        self.sub_url =
            Url::parse(&mock_server.url(format!("{subscribe_url_path}?token={token}"))).expect("不合法的订阅地址");

        mock_server
            .mock_async(|when, then| {
                when.method(POST).path(self.login_url_api().api);
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
                when.method(GET).path(self.get_sub_url_api().api);
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

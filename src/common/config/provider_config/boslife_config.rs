use crate::common::config::provider_config::{Api, ApiConfig, Credential, Headers, Provider, ProviderConfig};
use url::Url;

impl ProviderConfig {
    pub fn boslife_template() -> Self {
        Self {
            provider: Provider::BosLife,
            sub_url: Url::parse("http://127.0.0.1:8080/subscription?token=bppleman").expect("不合法的订阅地址"),
            api_config: ApiConfig {
                host: Url::parse("https://www.blnew.com").expect("不合法的 API 地址"),
                prefix: "/proxy/".to_string(),
                headers: Headers(
                    [
                        ("Accept", "application/json"),
                        ("Content-Type", "application/json"),
                        ("User-Agent", concat!("Convertor/", env!("CARGO_PKG_VERSION"))),
                        ("Authorization", "optional"),
                        ("Cookie", "optional"),
                        (
                            "sec-ch-ua",
                            r#""Not)A;Brand";v="8", "Chromium";v="138", "Google Chrome";v="138""#,
                        ),
                    ]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                ),
                credential: Credential {
                    username: "boslife.username".to_string(),
                    password: "boslife.password".to_string(),
                },
                login_api: Api {
                    path: "passport/auth/login".to_string(),
                    json_path: "$.data.auth_data".to_string(),
                },
                get_sub_api: Api {
                    path: "user/getSubscribe".to_string(),
                    json_path: "$.data.subscribe_url".to_string(),
                },
                reset_sub_api: Api {
                    path: "user/resetSubscribe".to_string(),
                    json_path: "$.data".to_string(),
                },
                sub_logs_api: Some(Api {
                    path: "user/stat/getSubscribeLog".to_string(),
                    json_path: "$.data.subscribe_url".to_string(),
                }),
            },
        }
    }
}

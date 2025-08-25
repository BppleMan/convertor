use crate::common::cache::{
    CACHED_AUTH_TOKEN_KEY, CACHED_PROFILE_KEY, CACHED_SUB_LOGS_KEY, CACHED_SUB_URL_KEY, Cache, CacheKey,
};
use crate::common::config::provider_config::ApiConfig;
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::ext::NonEmptyOptStr;
use crate::provider_api::boslife_api::BosLifeLogs;
use color_eyre::Report;
use color_eyre::eyre::{Context, eyre};
use headers::UserAgent;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Request};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

pub(super) trait ProviderApiTrait {
    fn api_config(&self) -> &ApiConfig;

    fn build_raw_url(&self, client: ProxyClient) -> Url;

    fn client(&self) -> &reqwest::Client;

    fn get_raw_profile_request(&self, raw_sub_url: Url, user_agent: UserAgent) -> color_eyre::Result<Request> {
        self.client()
            .request(Method::GET, raw_sub_url)
            .header("User-Agent", user_agent.as_str())
            .build()
            .wrap_err("构建 get_raw_profile 请求失败")
    }

    fn login_request(&self) -> color_eyre::Result<Request>;

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn cached_profile(&self) -> &Cache<String, String>;

    fn cached_auth_token(&self) -> &Cache<String, String>;

    fn cached_sub_url(&self) -> &Cache<String, String>;

    fn cached_sub_logs(&self) -> &Cache<String, BosLifeLogs>;

    async fn execute<B, F, R>(&self, build_request: B, trans_response: F) -> color_eyre::Result<R>
    where
        B: Fn() -> color_eyre::Result<Request>,
        F: Fn(String) -> color_eyre::Result<R>,
    {
        let mut request = build_request()?;
        self.api_config().headers.iter().for_each(|(key, value)| {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_str(key.as_str()),
                HeaderValue::from_str(value.as_str()),
            ) {
                request.headers_mut().insert(name, value);
            }
        });
        let response = self.client().execute(request).await?;
        let response_status = response.status();
        let response_headers = response.headers().clone();
        let response_text = response.text().await.wrap_err("获取 response_text 失败")?;
        if response_status.is_success() {
            Ok(trans_response(response_text)?)
        } else {
            use std::fmt::Write;
            let mut error = String::new();
            writeln!(&mut error, "Status: {}", response_status)?;
            writeln!(&mut error, "Headers:")?;
            for (key, value) in response_headers.iter() {
                writeln!(&mut error, "\t{}: {:?}", key, value)?;
            }
            writeln!(&mut error, "Body:")?;
            for line in response_text.lines() {
                writeln!(&mut error, "\t{}", line)?;
            }
            Err(eyre!(error))
        }
    }

    async fn get_raw_profile(&self, client: ProxyClient, user_agent: UserAgent) -> color_eyre::Result<String> {
        let raw_sub_url = self.build_raw_url(client);
        let result = self
            .cached_profile()
            .try_get_with(
                CacheKey::new(CACHED_PROFILE_KEY, raw_sub_url.to_string(), Some(client)),
                async {
                    self.execute(
                        || self.get_raw_profile_request(raw_sub_url.clone(), user_agent.clone()),
                        |text| Ok(text),
                    )
                    .await
                    .wrap_err("获取原始订阅文件失败")
                    .map_err(|e| format!("{e:?}"))
                },
            )
            .await
            .map_err(|e| eyre!(e))?;
        Ok(result)
    }

    async fn login(&self) -> color_eyre::Result<String> {
        if let Some(auth_token) = self.api_config().headers.get("Authorization").filter_non_empty() {
            return Ok(auth_token.to_string());
        }
        let result = self
            .cached_auth_token()
            .try_get_with(
                CacheKey::new(
                    CACHED_AUTH_TOKEN_KEY,
                    self.api_config().login_api.path.to_string(),
                    None,
                ),
                async {
                    self.execute(
                        || self.login_request(),
                        |text| {
                            let json_path = &self.api_config().login_api.json_path;
                            jsonpath_lib::select_as::<String>(&text, json_path)
                                .wrap_err(format!("无法选择 json_path: {json_path}"))?
                                .into_iter()
                                .next()
                                .ok_or(eyre!("未选择到任何内容 json_path: {json_path}"))
                        },
                    )
                    .await
                    .wrap_err("登录服务商失败")
                    .map_err(|e| format!("{e:?}"))
                },
            )
            .await
            .map_err(|e| eyre!(e))?;
        Ok(result)
    }

    async fn get_sub_url(&self) -> color_eyre::Result<Url> {
        let result = self
            .cached_sub_url()
            .try_get_with(
                CacheKey::new(CACHED_SUB_URL_KEY, self.api_config().get_sub_api.path.to_string(), None),
                async {
                    let auth_token = self.login().await.map_err(|e| format!("{e:?}"))?;
                    self.execute(
                        || self.get_sub_request(auth_token.clone()),
                        |text| {
                            let json_path = &self.api_config().get_sub_api.json_path;
                            jsonpath_lib::select_as::<String>(&text, json_path)
                                .wrap_err(format!("无法选择 json_path: {json_path}"))?
                                .into_iter()
                                .next()
                                .ok_or(eyre!("未选择到任何内容 json_path: {json_path}"))
                        },
                    )
                    .await
                    .wrap_err("获取原始订阅链接失败")
                    .map_err(|e| format!("{e:?}"))
                },
            )
            .await
            .map_err(|e| eyre!(e))?;
        let result = Url::parse(result.as_str())?;
        Ok(result)
    }

    async fn reset_sub_url(&self) -> color_eyre::Result<Url> {
        let auth_token = self.login().await?;
        let response = self
            .execute(
                || self.reset_sub_request(auth_token.clone()),
                |text| {
                    let json_path = &self.api_config().reset_sub_api.json_path;
                    jsonpath_lib::select_as::<String>(&text, json_path)
                        .wrap_err(format!("无法选择 json_path: {json_path}"))?
                        .into_iter()
                        .next()
                        .ok_or(eyre!("未选择到任何内容 json_path: {json_path}"))
                },
            )
            .await?;
        let url = Url::parse(&response)?;
        Ok(url)
    }

    async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeLogs> {
        let Some(api) = self.api_config().sub_logs_api.as_ref() else {
            return Err(eyre!("该提供商未配置订阅日志接口"));
        };
        let result = self
            .cached_sub_logs()
            .try_get_with(CacheKey::new(CACHED_SUB_LOGS_KEY, api.path.to_string(), None), async {
                let auth_token = self.login().await.map_err(|e| format!("{e:?}"))?;
                self.execute(
                    || self.get_sub_logs_request(auth_token.clone()),
                    |text| {
                        let json_path = &api.json_path;
                        jsonpath_lib::select_as::<BosLifeLogs>(&text, json_path)
                            .wrap_err(format!("无法选择 json_path: {json_path}"))?
                            .into_iter()
                            .next()
                            .ok_or(eyre!("未选择到任何内容 json_path: {json_path}"))
                    },
                )
                .await
                .wrap_err("获取订阅日志失败")
                .map_err(|e| format!("{e:?}"))
            })
            .await
            .map_err(|e| eyre!(e))?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct ReportWrapper(pub Report);

impl Display for ReportWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ReportWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

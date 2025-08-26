use crate::common::cache::{
    CACHED_AUTH_TOKEN_KEY, CACHED_PROFILE_KEY, CACHED_SUB_LOGS_KEY, CACHED_SUB_URL_KEY, Cache, CacheKey,
};
use crate::common::config::provider_config::ApiConfig;
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::ext::NonEmptyOptStr;
use crate::provider_api::api_response::{ApiFailed, ApiResponse};
use crate::provider_api::boslife_api::BosLifeLogs;
use color_eyre::eyre::{Context, eyre};
use headers::UserAgent;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Request};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

#[async_trait::async_trait]
pub trait ProviderApiTrait: Clone + Send {
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

    async fn execute<Req, Resp, ReqFut, RepFut, R>(
        &self,
        request_future: Req,
        response_future: Resp,
    ) -> color_eyre::Result<ApiResponse<R>>
    where
        R: Debug + Clone + Send,
        Req: FnOnce() -> ReqFut + Send,
        Resp: FnOnce(String) -> RepFut + Send,
        ReqFut: Future<Output = color_eyre::Result<Request>> + Send,
        RepFut: Future<Output = color_eyre::Result<R>> + Send,
    {
        let mut request = request_future().await?;
        self.api_config().headers.iter().for_each(|(key, value)| {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_str(key.as_str()),
                HeaderValue::from_str(value.as_str()),
            ) {
                request.headers_mut().insert(name, value);
            }
        });
        let method = request.method().clone();
        let url = request.url().clone();
        let response = self.client().execute(request).await?;
        let response_status = response.status();
        let response_headers = response.headers().clone();
        let response_text = response.text().await.wrap_err("获取 response_text 失败")?;
        if response_status.is_success() {
            let resp = response_future(response_text).await?;
            Ok(ApiResponse::Success(resp))
        } else {
            let failed = ApiFailed {
                url,
                method,
                status: response_status,
                headers: response_headers,
                body: response_text,
            };
            Ok(ApiResponse::Failed(Box::new(failed)))
        }
    }

    async fn get_raw_profile(&self, client: ProxyClient, user_agent: UserAgent) -> color_eyre::Result<String> {
        let raw_sub_url = self.build_raw_url(client);
        let key = CacheKey::new(CACHED_PROFILE_KEY, raw_sub_url.to_string(), Some(client));
        let result = self
            .cached_profile()
            .try_get_with(key, async {
                match self
                    .execute(
                        || async { self.get_raw_profile_request(raw_sub_url.clone(), user_agent.clone()) },
                        |text| async { Ok(text) },
                    )
                    .await
                    .wrap_err("获取原始订阅文件失败")
                    .map_err(|e| format!("{e:?}"))?
                {
                    ApiResponse::Success(raw_profile) => Ok(raw_profile),
                    ApiResponse::Failed(failed) => Err(format!("{failed}")),
                }
            })
            .await
            .map_err(|e| eyre!(e))?;
        Ok(result)
    }

    async fn login(&self) -> color_eyre::Result<String> {
        if let Some(auth_token) = self.api_config().headers.get("Authorization").filter_non_empty() {
            return Ok(auth_token.to_string());
        }
        let key = CacheKey::new(CACHED_AUTH_TOKEN_KEY, self.api_config().login_url().to_string(), None);
        let result = self
            .cached_auth_token()
            .try_get_with(key, async {
                match self
                    .execute(
                        || async { self.login_request() },
                        |text| async move {
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
                    .map_err(|e| format!("{e:?}"))?
                {
                    ApiResponse::Success(auth_token) => Ok(auth_token),
                    ApiResponse::Failed(failed) => Err(format!("{failed}")),
                }
            })
            .await
            .map_err(|e| eyre!(e))?;
        Ok(result)
    }

    async fn get_sub_url(&self) -> color_eyre::Result<Url> {
        let key = CacheKey::new(CACHED_SUB_URL_KEY, self.api_config().get_sub_url().to_string(), None);
        let result = self
            .cached_sub_url()
            .try_get_with(key, async {
                match self
                    .execute(
                        || async {
                            let auth_token = self.login().await?;
                            self.get_sub_request(auth_token)
                        },
                        |text| async move {
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
                    .map_err(|e| format!("{e:?}"))?
                {
                    ApiResponse::Success(sub_url) => Ok(sub_url),
                    ApiResponse::Failed(failed) => Err(format!("{failed}")),
                }
            })
            .await
            .map_err(|e: Arc<String>| eyre!(e))?;

        let result = Url::parse(result.as_str())?;
        Ok(result)
    }

    async fn reset_sub_url(&self) -> color_eyre::Result<Url> {
        let response = self
            .execute(
                || async {
                    let auth_token = self.login().await?;
                    self.reset_sub_request(auth_token)
                },
                |text| async move {
                    let json_path = &self.api_config().reset_sub_api.json_path;
                    jsonpath_lib::select_as::<String>(&text, json_path)
                        .wrap_err(format!("无法选择 json_path: {json_path}"))?
                        .into_iter()
                        .next()
                        .ok_or(eyre!("未选择到任何内容 json_path: {json_path}"))
                },
            )
            .await?;
        match response {
            ApiResponse::Success(url) => Ok(Url::parse(&url)?),
            ApiResponse::Failed(e) => Err(eyre!(e)),
        }
    }

    async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeLogs> {
        let sub_logs_url = self.api_config().sub_logs_url().ok_or(eyre!("订阅日志接口未配置"))?;
        let key = CacheKey::new(CACHED_SUB_LOGS_KEY, sub_logs_url.to_string(), None);
        let result = self
            .cached_sub_logs()
            .try_get_with(key, async {
                match self
                    .execute(
                        || async {
                            let auth_token = self.login().await?;
                            self.get_sub_logs_request(auth_token)
                        },
                        |text| async move {
                            let json_path = self
                                .api_config()
                                .sub_logs_api
                                .as_ref()
                                .map(|a| &a.json_path)
                                .ok_or(eyre!("订阅日志接口未配置 json_path"))?
                                .clone();
                            jsonpath_lib::select_as::<BosLifeLogs>(&text, &json_path)
                                .wrap_err(format!("无法选择 json_path: {json_path}"))?
                                .into_iter()
                                .next()
                                .ok_or(eyre!("未选择到任何内容 json_path: {json_path}"))
                        },
                    )
                    .await
                    .wrap_err("获取订阅日志失败")
                    .map_err(|e| format!("{e:?}"))?
                {
                    ApiResponse::Success(logs) => Ok(logs),
                    ApiResponse::Failed(failed) => Err(format!("{failed}")),
                }
            })
            .await
            .map_err(|e| eyre!(e))?;
        Ok(result)
    }
}

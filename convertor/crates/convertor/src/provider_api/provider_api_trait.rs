use crate::common::cache::{
    Cache, CacheKey, CACHED_AUTH_TOKEN_KEY, CACHED_PROFILE_KEY, CACHED_SUB_LOGS_KEY, CACHED_SUB_URL_KEY,
};
use crate::common::ext::NonEmptyOptStr;
use crate::config::client_config::ProxyClient;
use crate::config::provider_config::ApiConfig;
use crate::error::ProviderApiError;
use crate::provider_api::api_response::ApiFailed;
use crate::provider_api::boslife_api::BosLifeLogs;
use headers::UserAgent;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Request};
use std::fmt::Debug;
use std::str::FromStr;
use tracing::error;
use url::Url;

type Result<T> = core::result::Result<T, ProviderApiError>;

#[async_trait::async_trait]
pub trait ProviderApiTrait: Clone + Send {
    fn api_config(&self) -> &ApiConfig;

    fn build_raw_url(&self, client: ProxyClient) -> Url;

    fn client(&self) -> &reqwest::Client;

    fn get_raw_profile_request(&self, raw_sub_url: Url, user_agent: UserAgent) -> Result<Request> {
        self.client()
            .request(Method::GET, raw_sub_url)
            .header("user-agent", user_agent.as_str())
            .build()
            .map_err(|e| ProviderApiError::BuildRequestError {
                name: "[get_raw_profile]".to_string(),
                source: e,
            })
    }

    fn login_request(&self) -> Result<Request>;

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> Result<Request>;

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> Result<Request>;

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> Result<Request>;

    fn cached_profile(&self) -> &Cache<String, String>;

    fn cached_auth_token(&self) -> &Cache<String, String>;

    fn cached_sub_url(&self) -> &Cache<String, String>;

    fn cached_sub_logs(&self) -> &Cache<String, BosLifeLogs>;

    async fn execute<Req, Resp, ReqFut, RepFut, R>(
        &self,
        name: String,
        request_future: Req,
        response_future: Resp,
    ) -> Result<R>
    where
        R: Debug + Clone + Send,
        Req: FnOnce() -> ReqFut + Send,
        Resp: FnOnce(String) -> RepFut + Send,
        ReqFut: Future<Output = Result<Request>> + Send,
        RepFut: Future<Output = Result<R>> + Send,
    {
        let mut request = request_future().await?;
        self.api_config().headers.iter().for_each(|(key, value)| {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_str(key.as_str()),
                HeaderValue::from_str(value.as_str()),
            ) {
                if !name.as_str().is_empty() && !value.as_bytes().is_empty() {
                    request.headers_mut().insert(name, value);
                }
            }
        });
        let method = request.method().clone();
        let url = request.url().clone();
        let request_headers = request.headers().clone();
        let response = self
            .client()
            .execute(request)
            .await
            .map_err(|e| ProviderApiError::ResponseError {
                name: format!("[{}] {}", method.as_str().to_uppercase(), url),
                source: e,
            })?;
        let response_status = response.status();
        let response_headers = response.headers().clone();
        let response_text = response.text().await.map_err(|e| ProviderApiError::ResponseError {
            name: format!("[{}] {}", method.as_str().to_uppercase(), url),
            source: e,
        })?;
        if response_status.is_success() {
            let resp = response_future(response_text).await?;
            Ok(resp)
        } else {
            let failed = ApiFailed {
                request_url: url,
                request_method: method,
                request_headers,
                response_status,
                response_headers,
                response_body: response_text,
            };
            error!("{}", failed);
            Err(ProviderApiError::ApiFailed {
                name,
                source: Box::new(failed),
            })
        }
    }

    async fn get_raw_profile(&self, client: ProxyClient, user_agent: UserAgent) -> Result<String> {
        let raw_sub_url = self.build_raw_url(client);
        let key = CacheKey::new(CACHED_PROFILE_KEY, raw_sub_url.to_string(), Some(client));
        let result = self
            .cached_profile()
            .try_get_with(key, async {
                self.execute(
                    "获取原始订阅内容".to_string(),
                    || async { self.get_raw_profile_request(raw_sub_url.clone(), user_agent.clone()) },
                    |text| async { Ok(text) },
                )
                .await
            })
            .await?;
        Ok(result)
    }

    async fn login(&self) -> Result<String> {
        if let Some(auth_token) = self.api_config().headers.get("Authorization").filter_non_empty() {
            return Ok(auth_token.to_string());
        }
        let key = CacheKey::new(CACHED_AUTH_TOKEN_KEY, self.api_config().login_url().to_string(), None);
        let result = self
            .cached_auth_token()
            .try_get_with(key, async {
                self.execute(
                    "登录服务商".to_string(),
                    || async { self.login_request() },
                    |text| async move {
                        let json_path = &self.api_config().login_api.json_path;
                        jsonpath_lib::select_as::<String>(&text, json_path)
                            .map_err(|e| ProviderApiError::JsonPathError {
                                name: "[登录服务商]".to_string(),
                                path: json_path.clone(),
                                source: e,
                            })?
                            .into_iter()
                            .next()
                            .ok_or(ProviderApiError::JsonPathNotFound {
                                name: "[登录服务商]".to_string(),
                                path: json_path.clone(),
                            })
                    },
                )
                .await
            })
            .await
            .map_err(|e| ProviderApiError::InnerError(e))?;
        Ok(result)
    }

    async fn get_sub_url(&self) -> Result<Url> {
        let key = CacheKey::new(CACHED_SUB_URL_KEY, self.api_config().get_sub_url().to_string(), None);
        let result = self
            .cached_sub_url()
            .try_get_with(key, async {
                self.execute(
                    "获取原始订阅链接".to_string(),
                    || async {
                        let auth_token = self.login().await?;
                        self.get_sub_request(auth_token)
                    },
                    |text| async move {
                        let json_path = &self.api_config().get_sub_api.json_path;
                        jsonpath_lib::select_as::<String>(&text, json_path)
                            .map_err(|e| ProviderApiError::JsonPathError {
                                name: "[获取原始订阅链接]".to_string(),
                                path: json_path.clone(),
                                source: e,
                            })?
                            .into_iter()
                            .next()
                            .ok_or(ProviderApiError::JsonPathNotFound {
                                name: "[获取原始订阅链接]".to_string(),
                                path: json_path.clone(),
                            })
                    },
                )
                .await
            })
            .await?;

        let result = Url::parse(result.as_str())?;
        Ok(result)
    }

    async fn reset_sub_url(&self) -> Result<Url> {
        let response = self
            .execute(
                "重置订阅链接".to_string(),
                || async {
                    let auth_token = self.login().await?;
                    self.reset_sub_request(auth_token)
                },
                |text| async move {
                    let json_path = &self.api_config().reset_sub_api.json_path;
                    jsonpath_lib::select_as::<String>(&text, json_path)
                        .map_err(|e| ProviderApiError::JsonPathError {
                            name: "[重置订阅链接]".to_string(),
                            path: json_path.clone(),
                            source: e,
                        })?
                        .into_iter()
                        .next()
                        .ok_or(ProviderApiError::JsonPathNotFound {
                            name: "[重置订阅链接]".to_string(),
                            path: json_path.clone(),
                        })
                },
            )
            .await?;
        Ok(Url::parse(&response)?)
    }

    async fn get_sub_logs(&self) -> Result<BosLifeLogs> {
        let sub_logs_url = self
            .api_config()
            .sub_logs_url()
            .ok_or(ProviderApiError::Other("订阅日志接口未配置".to_string()))?;
        let key = CacheKey::new(CACHED_SUB_LOGS_KEY, sub_logs_url.to_string(), None);
        let result = self
            .cached_sub_logs()
            .try_get_with(key, async {
                self.execute(
                    "获取订阅日志".to_string(),
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
                            .ok_or(ProviderApiError::Other("订阅日志接口未配置 json_path".to_string()))?
                            .clone();
                        jsonpath_lib::select_as::<BosLifeLogs>(&text, &json_path)
                            .map_err(|e| ProviderApiError::JsonPathError {
                                name: "[获取订阅日志]".to_string(),
                                path: json_path.clone(),
                                source: e,
                            })?
                            .into_iter()
                            .next()
                            .ok_or(ProviderApiError::JsonPathNotFound {
                                name: "[获取订阅日志]".to_string(),
                                path: json_path.clone(),
                            })
                    },
                )
                .await
            })
            .await?;
        Ok(result)
    }
}

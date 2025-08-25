use crate::common::cache::{
    CACHED_AUTH_TOKEN_KEY, CACHED_PROFILE_KEY, CACHED_SUB_LOGS_KEY, CACHED_SUB_URL_KEY, Cache, CacheKey,
};
use crate::common::config::provider_config::ApiConfig;
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::ext::NonEmptyOptStr;
use crate::provider_api::boslife_api::BosLifeLogs;
use color_eyre::eyre::{Context, eyre};
use headers::UserAgent;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Request, Response};
use std::str::FromStr;
use url::Url;

pub(super) trait ProviderApiTrait {
    fn api_config(&self) -> &ApiConfig;

    fn build_raw_url(&self, client: ProxyClient) -> Url;

    fn client(&self) -> &reqwest::Client;

    fn login_request(&self) -> color_eyre::Result<Request>;

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn cached_profile(&self) -> &Cache<String, String>;

    fn cached_auth_token(&self) -> &Cache<String, String>;

    fn cached_sub_url(&self) -> &Cache<String, String>;

    fn cached_sub_logs(&self) -> &Cache<String, BosLifeLogs>;

    async fn execute(&self, mut request: Request) -> color_eyre::Result<Response> {
        self.api_config().headers.iter().for_each(|(key, value)| {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_str(key.as_str()),
                HeaderValue::from_str(value.as_str()),
            ) {
                request.headers_mut().insert(name, value);
            }
        });
        Ok(self.client().execute(request).await?)
    }

    async fn get_raw_profile(&self, client: ProxyClient, user_agent: UserAgent) -> color_eyre::Result<String> {
        let raw_sub_url = self.build_raw_url(client);
        self.cached_profile()
            .try_get_with(
                CacheKey::new(CACHED_PROFILE_KEY, raw_sub_url.to_string(), Some(client)),
                async {
                    let request = self
                        .client()
                        .request(Method::GET, raw_sub_url)
                        .header("User-Agent", user_agent.as_str())
                        .build()?;
                    let response = self.execute(request).await?;
                    let status = format!("响应状态: {:?}", response.status());
                    let headers = format!("响应头: {:?}", response.headers());
                    if response.status().is_success() {
                        response.text().await.map_err(Into::into)
                    } else {
                        let body = response.bytes().await?;
                        let content = String::from_utf8_lossy(&body).to_string();
                        let content = format!("{}\n{}\n响应体: {}", status, headers, content);
                        let error_report = eyre!("获取原始订阅文件失败:\n{}", content);
                        Err(error_report)
                    }
                },
            )
            .await
            .map_err(|e| eyre!(e))
    }

    async fn login(&self) -> color_eyre::Result<String> {
        if let Some(auth_token) = self.api_config().headers.get("Authorization").filter_non_empty() {
            return Ok(auth_token.to_string());
        }
        self.cached_auth_token()
            .try_get_with(
                CacheKey::new(
                    CACHED_AUTH_TOKEN_KEY,
                    self.api_config().login_api.path.to_string(),
                    None,
                ),
                async {
                    let request = self.login_request()?;
                    let response = self.execute(request).await?;
                    let json_path = &self.api_config().login_api.json_path;
                    if response.status().is_success() {
                        let json_response = response.text().await?;
                        let auth_token = jsonpath_lib::select_as(&json_response, &json_path)
                            .wrap_err_with(|| format!("无法选择 json_path: {}", &json_path))?
                            .remove(0);
                        Ok(auth_token)
                    } else {
                        let status = response.status();
                        let error_report = response
                            .text()
                            .await
                            .map(|msg| eyre!(msg))
                            .wrap_err("获取错误信息失败")
                            .unwrap_or_else(|e| e)
                            .wrap_err(format!("登录服务商失败: {}", status));
                        Err(error_report)
                    }
                },
            )
            .await
            .map_err(|e| eyre!(e))
    }

    async fn get_sub_url(&self) -> color_eyre::Result<Url> {
        self.cached_sub_url()
            .try_get_with(
                CacheKey::new(CACHED_SUB_URL_KEY, self.api_config().get_sub_api.path.to_string(), None),
                async {
                    let auth_token = self.login().await?;
                    let request = self.get_sub_request(auth_token)?;
                    let response = self.execute(request).await?;
                    if response.status().is_success() {
                        let json_response = response.text().await?;
                        let json_path = &self.api_config().get_sub_api.json_path;
                        let url_str: String = jsonpath_lib::select_as(&json_response, json_path)
                            .wrap_err_with(|| format!("failed to select json_path: {json_path}"))?
                            .remove(0);
                        Ok(url_str)
                    } else {
                        let status = response.status();
                        let error_report = response
                            .text()
                            .await
                            .map(|msg| eyre!(msg))
                            .wrap_err("获取错误信息失败")
                            .unwrap_or_else(|e| e)
                            .wrap_err(format!("请求服务商原始订阅链接失败: {}", status));
                        Err(error_report)
                    }
                },
            )
            .await
            .map(|s| Ok(Url::parse(&s)?))
            .map_err(|e| eyre!(e))?
    }

    async fn reset_sub_url(&self) -> color_eyre::Result<Url> {
        let auth_token = self.login().await?;
        let request = self.reset_sub_request(auth_token)?;
        let response = self.execute(request).await?;
        if response.status().is_success() {
            let json_path = &self.api_config().reset_sub_api.json_path;
            let json_response = response.text().await?;
            let url_str: String = jsonpath_lib::select_as(&json_response, json_path)
                .wrap_err_with(|| format!("failed to select json_path: {}", &json_path))?
                .remove(0);
            Url::parse(&url_str).map_err(|e| e.into())
        } else {
            let status = response.status();
            let error_report = response
                .text()
                .await
                .map(|msg| eyre!(msg))
                .wrap_err("获取错误信息失败")
                .unwrap_or_else(|e| e)
                .wrap_err(format!("重置原始订阅链接失败: {}", status));
            Err(error_report)
        }
    }

    async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeLogs> {
        let Some(api) = self.api_config().sub_logs_api.as_ref() else {
            return Err(eyre!("该提供商未配置订阅日志接口"));
        };
        self.cached_sub_logs()
            .try_get_with(CacheKey::new(CACHED_SUB_LOGS_KEY, api.path.to_string(), None), async {
                let auth_token = self.login().await?;
                let request = self.get_sub_logs_request(auth_token)?;
                let response = self.execute(request).await?;
                if response.status().is_success() {
                    let response = response.text().await?;
                    let json_path = &api.json_path;
                    let response: BosLifeLogs = jsonpath_lib::select_as(&response, &json_path)?.remove(0);
                    Ok(response)
                } else {
                    Err(eyre!("Get subscription log failed: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

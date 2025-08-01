use crate::api::boslife_sub_log::BosLifeSubLogs;
use crate::common::cache::{
    CACHED_AUTH_TOKEN_KEY, CACHED_PROFILE_KEY, CACHED_SUB_LOGS_KEY, CACHED_UNI_SUB_URL_KEY, Cache, CacheKey,
};
use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::request::RequestConfig;
use crate::common::config::sub_provider::ApiConfig;
use axum::http::HeaderValue;
use color_eyre::eyre::{Context, eyre};
use moka::future::Cache as MokaCache;
use reqwest::header::HeaderName;
use reqwest::{Method, Request, Response};
use std::str::FromStr;
use url::Url;

pub(crate) trait SubProviderApi {
    fn common_request_config(&self) -> Option<&RequestConfig>;

    fn api_host(&self) -> &Url;

    fn login_api(&self) -> &ApiConfig;

    fn get_sub_url_api(&self) -> &ApiConfig;

    fn reset_sub_url_api(&self) -> &ApiConfig;

    fn get_sub_logs_api(&self) -> &ApiConfig;

    fn build_raw_sub_url(&self, client: ProxyClient) -> Url;

    fn client(&self) -> &reqwest::Client;

    fn login_request(&self) -> color_eyre::Result<Request>;

    fn get_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn reset_sub_url_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn cached_auth_token(&self) -> &MokaCache<String, String>;

    fn cached_profile(&self) -> &Cache<Url, String>;

    fn cached_sub_url(&self) -> &Cache<Url, String>;

    fn cached_sub_logs(&self) -> &Cache<Url, BosLifeSubLogs>;

    async fn execute(&self, mut request: Request) -> color_eyre::Result<Response> {
        if let Some(request_config) = self.common_request_config() {
            if let Some(cookie) = request_config.cookie.as_ref() {
                request.headers_mut().insert("Cookie", cookie.parse()?);
            }
            if let Some(ua) = request_config.user_agent.as_ref() {
                request.headers_mut().insert("User-Agent", ua.parse()?);
            }
            request_config.headers.iter().for_each(|(key, value)| {
                if let (Ok(name), Ok(value)) = (
                    HeaderName::from_str(key.as_str()),
                    HeaderValue::from_str(value.as_str()),
                ) {
                    request.headers_mut().insert(name, value);
                }
            });
        }
        Ok(self.client().execute(request).await?)
    }

    async fn get_raw_profile(&self, client: ProxyClient) -> color_eyre::Result<String> {
        let raw_sub_url = self.build_raw_sub_url(client);
        let key = CacheKey::new(CACHED_PROFILE_KEY, raw_sub_url.clone(), Some(client));
        self.cached_profile()
            .try_get_with(key, async {
                let request = self.client().request(Method::GET, raw_sub_url).build()?;
                let response = self.execute(request).await?;
                if response.status().is_success() {
                    response.text().await.map_err(Into::into)
                } else {
                    Err(eyre!("Get raw profile failed: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }

    async fn login(&self) -> color_eyre::Result<String> {
        if let Some(Some(auth_token)) = &self.common_request_config().map(|r| &r.auth_token) {
            return Ok(auth_token.clone());
        }
        self.cached_auth_token()
            .try_get_with(format!("{}_{}", CACHED_AUTH_TOKEN_KEY, self.api_host()), async {
                let request = self.login_request()?;
                let response = self.execute(request).await?;
                let json_path = &self.login_api().json_path;
                if response.status().is_success() {
                    let json_response = response.text().await?;
                    let auth_token = jsonpath_lib::select_as(&json_response, &json_path)
                        .wrap_err_with(|| format!("failed to select json_path: {}", &json_path))?
                        .remove(0);
                    Ok(auth_token)
                } else {
                    Err(eyre!("登陆服务商失败: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }

    async fn get_uni_sub_url(&self) -> color_eyre::Result<Url> {
        self.cached_sub_url()
            .try_get_with(
                CacheKey::new(CACHED_UNI_SUB_URL_KEY, self.api_host().clone(), None),
                async {
                    let auth_token = self.login().await?;
                    let request = self.get_sub_url_request(auth_token)?;
                    let response = self.execute(request).await?;
                    if response.status().is_success() {
                        let json_response = response.text().await?;
                        let json_path = &self.get_sub_url_api().json_path;
                        let url_str: String = jsonpath_lib::select_as(&json_response, json_path)
                            .wrap_err_with(|| format!("failed to select json_path: {json_path}"))?
                            .remove(0);
                        Ok(url_str)
                    } else {
                        Err(eyre!("请求服务商原始订阅链接失败: {}", response.status(),))
                    }
                },
            )
            .await
            .map(|s| Ok(Url::parse(&s)?))
            .map_err(|e| eyre!(e))?
    }

    async fn reset_uni_sub_url(&self) -> color_eyre::Result<Url> {
        let auth_token = self.login().await?;
        let request = self.reset_sub_url_request(auth_token)?;
        let response = self.execute(request).await?;
        if response.status().is_success() {
            let json_path = &self.reset_sub_url_api().json_path;
            let json_response = response.text().await?;
            let url_str: String = jsonpath_lib::select_as(&json_response, &json_path)
                .wrap_err_with(|| format!("failed to select json_path: {}", &json_path))?
                .remove(0);
            Url::parse(&url_str).map_err(|e| e.into())
        } else {
            Err(eyre!("Reset raw subscription URL failed: {}", response.status()))
        }
    }

    async fn get_sub_logs(&self) -> color_eyre::Result<BosLifeSubLogs> {
        self.cached_sub_logs()
            .try_get_with(
                CacheKey::new(CACHED_SUB_LOGS_KEY, self.api_host().clone(), None),
                async {
                    let auth_token = self.login().await?;
                    let request = self.get_sub_logs_request(auth_token)?;
                    let response = self.execute(request).await?;
                    if response.status().is_success() {
                        let response = response.text().await?;
                        let json_path = &self.get_sub_logs_api().json_path;
                        let response: BosLifeSubLogs = jsonpath_lib::select_as(&response, &json_path)?.remove(0);
                        Ok(response)
                    } else {
                        Err(eyre!("Get subscription log failed: {}", response.status()))
                    }
                },
            )
            .await
            .map_err(|e| eyre!(e))
    }
}

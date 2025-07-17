use crate::cache::{
    CACHED_AUTH_TOKEN_KEY, CACHED_PROFILE_KEY, CACHED_RAW_SUB_URL_KEY, CACHED_SUB_LOGS_KEY, Cache, CacheKey,
};
use crate::client::Client;
use crate::service_provider::api::subscription_log::SubscriptionLogs;
use crate::service_provider::config::ServiceConfig;
use axum::http::Method;
use color_eyre::eyre::{Context, eyre};
use moka::future::Cache as MokaCache;
use reqwest::{Request, Response};
use url::Url;

pub(crate) trait ServiceApiCommon {
    fn config(&self) -> &ServiceConfig;

    fn client(&self) -> &reqwest::Client;

    fn login_request(&self) -> color_eyre::Result<Request>;

    fn get_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn reset_sub_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn get_sub_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn cached_auth_token(&self) -> &MokaCache<String, String>;

    fn cached_profile(&self) -> &Cache<Url, String>;

    fn cached_raw_sub_url(&self) -> &Cache<Url, String>;

    fn cached_sub_logs(&self) -> &Cache<Url, SubscriptionLogs>;

    async fn execute(&self, mut request: Request) -> color_eyre::Result<Response> {
        request
            .headers_mut()
            .insert("User-Agent", concat!("convertor/", env!("CARGO_PKG_VERSION")).parse()?);
        Ok(self.client().execute(request).await?)
    }

    async fn get_raw_profile(&self, client: Client) -> color_eyre::Result<String> {
        let raw_sub_url = self.config().build_raw_sub_url(client)?;
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
        if !self.config().auth_token.is_empty() {
            return Ok(self.config().auth_token.clone());
        }
        self.cached_auth_token()
            .try_get_with(format!("{}_{}", CACHED_AUTH_TOKEN_KEY, self.config().api_host), async {
                let request = self.login_request()?;
                let response = self.execute(request).await?;
                if response.status().is_success() {
                    let json_response = response.text().await?;
                    let auth_token = jsonpath_lib::select_as(&json_response, &self.config().login_api.json_path)
                        .wrap_err_with(|| format!("failed to select json_path: {}", self.config().login_api.json_path))?
                        .remove(0);
                    Ok(auth_token)
                } else {
                    Err(eyre!("登陆服务商失败: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }

    async fn get_raw_sub_url(&self) -> color_eyre::Result<Url> {
        self.cached_raw_sub_url()
            .try_get_with(
                CacheKey::new(CACHED_RAW_SUB_URL_KEY, self.config().api_host.clone(), None),
                async {
                    let auth_token = self.login().await?;
                    let request = self.get_sub_request(auth_token)?;
                    let response = self.execute(request).await?;
                    if response.status().is_success() {
                        let json_response = response.text().await?;
                        let json_path = &self.config().get_sub_api.json_path;
                        let url_str: String = jsonpath_lib::select_as(&json_response, json_path)
                            .wrap_err_with(|| format!("failed to select json_path: {}", json_path))?
                            .remove(0);
                        Ok(url_str)
                    } else {
                        Err(eyre!("请求服务商订阅配置失败: {}", response.status(),))
                    }
                },
            )
            .await
            .map(|s| Ok(Url::parse(&s)?))
            .map_err(|e| eyre!(e))?
    }

    async fn reset_raw_sub_url(&self) -> color_eyre::Result<Url> {
        let auth_token = self.login().await?;
        let request = self.reset_sub_request(auth_token)?;
        let response = self.execute(request).await?;
        if response.status().is_success() {
            let json_response = response.text().await?;
            let url_str: String = jsonpath_lib::select_as(&json_response, &self.config().reset_sub_api.json_path)
                .wrap_err_with(|| format!("failed to select json_path: {}", self.config().reset_sub_api.json_path))?
                .remove(0);
            Url::parse(&url_str).map_err(|e| e.into())
        } else {
            Err(eyre!("Reset raw subscription URL failed: {}", response.status()))
        }
    }

    async fn get_sub_logs(&self) -> color_eyre::Result<SubscriptionLogs> {
        let cache_key = CacheKey::new(CACHED_SUB_LOGS_KEY, self.config().api_host.clone(), None);
        self.cached_sub_logs()
            .try_get_with(cache_key, async {
                let auth_token = self.login().await?;
                let request = self.get_sub_logs_request(auth_token)?;
                let response = self.execute(request).await?;
                if response.status().is_success() {
                    let response = response.text().await?;
                    let response: SubscriptionLogs =
                        jsonpath_lib::select_as(&response, &self.config().get_sub_logs_api.json_path)?.remove(0);
                    Ok(response)
                } else {
                    Err(eyre!("Get subscription log failed: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }
}

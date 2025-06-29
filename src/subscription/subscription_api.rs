use crate::subscription::subscription_config::ServiceConfig;
use crate::subscription::subscription_log::SubscriptionLog;
use color_eyre::eyre::{eyre, WrapErr};
use moka::future::Cache;
use reqwest::{Method, Request, Response, Url};

pub mod boslife_api;

pub const CACHED_AUTH_TOKEN_KEY: &str = "CACHED_AUTH_TOKEN";
pub const CACHED_PROFILE_KEY: &str = "CACHED_PROFILE";
pub const CACHED_RAW_SUBSCRIPTION_URL_KEY: &str = "CACHED_RAW_SUBSCRIPTION_URL";
pub const CACHED_SUBSCRIPTION_LOGS_KEY: &str = "CACHED_SUBSCRIPTION_LOGS";

/// 设置为 pub(crate) 可以避免 trait async fn 报错
pub(crate) trait ServiceApi {
    fn config(&self) -> &ServiceConfig;

    fn client(&self) -> &reqwest::Client;

    fn login_request(&self) -> color_eyre::Result<Request>;

    fn get_subscription_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn reset_subscription_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn get_subscription_logs_request(&self, auth_token: impl AsRef<str>) -> color_eyre::Result<Request>;

    fn cached_auth_token(&self) -> &Cache<String, String>;

    fn cached_profile(&self) -> &Cache<String, String>;

    fn cached_raw_subscription_url(&self) -> &Cache<String, Url>;

    fn cached_subscription_logs(&self) -> &Cache<String, Vec<SubscriptionLog>>;

    async fn execute(&self, mut request: Request) -> color_eyre::Result<Response> {
        request
            .headers_mut()
            .insert("User-Agent", concat!("convertor/", env!("CARGO_PKG_VERSION")).parse()?);
        Ok(self.client().execute(request).await?)
    }

    async fn login(&self) -> color_eyre::Result<String> {
        let request = self.login_request()?;
        self.cached_auth_token()
            .try_get_with(format!("{}_{}", CACHED_AUTH_TOKEN_KEY, request.url().as_str()), async {
                let response = self.execute(request).await?;
                if response.status().is_success() {
                    let json_response = response.text().await?;
                    let auth_token = jsonpath_lib::select_as(&json_response, &self.config().login_api.json_path)
                        .wrap_err_with(|| format!("failed to select json_path: {}", self.config().login_api.json_path))?
                        .remove(0);
                    Ok(auth_token)
                } else {
                    Err(eyre!("Login failed: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }

    async fn get_raw_profile(&self, url: Url) -> color_eyre::Result<String> {
        self.cached_profile()
            .try_get_with(format!("{}_{}", CACHED_PROFILE_KEY, url.as_str()), async {
                let request = self.client().request(Method::GET, url).build()?;
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

    async fn get_raw_subscription_url(&self) -> color_eyre::Result<Url> {
        let auth_token = self.login().await?;
        let request = self.get_subscription_request(auth_token)?;
        self.cached_raw_subscription_url()
            .try_get_with(
                format!("{}_{}", CACHED_RAW_SUBSCRIPTION_URL_KEY, request.url().as_str()),
                async {
                    let response = self.execute(request).await?;
                    if response.status().is_success() {
                        let json_response = response.text().await?;
                        let url_str: String =
                            jsonpath_lib::select_as(&json_response, &self.config().get_subscription_api.json_path)
                                .wrap_err_with(|| {
                                    format!(
                                        "failed to select json_path: {}",
                                        self.config().get_subscription_api.json_path
                                    )
                                })?
                                .remove(0);
                        Url::parse(&url_str).map_err(|e| e.into())
                    } else {
                        Err(eyre!("Get raw subscription URL failed: {}", response.status()))
                    }
                },
            )
            .await
            .map_err(|e| eyre!(e))
    }

    async fn reset_raw_subscription_url(&self) -> color_eyre::Result<Url> {
        let auth_token = self.login().await?;
        let request = self.reset_subscription_request(auth_token)?;
        let response = self.execute(request).await?;
        if response.status().is_success() {
            let json_response = response.text().await?;
            let url_str: String =
                jsonpath_lib::select_as(&json_response, &self.config().reset_subscription_api.json_path)
                    .wrap_err_with(|| {
                        format!(
                            "failed to select json_path: {}",
                            self.config().reset_subscription_api.json_path
                        )
                    })?
                    .remove(0);
            Url::parse(&url_str).map_err(|e| e.into())
        } else {
            Err(eyre!("Reset raw subscription URL failed: {}", response.status()))
        }
    }

    async fn get_subscription_logs(&self) -> color_eyre::Result<Vec<SubscriptionLog>> {
        let auth_token = self.login().await?;
        let request = self.get_subscription_logs_request(auth_token)?;
        self.cached_subscription_logs()
            .try_get_with(
                format!("{}_{}", CACHED_SUBSCRIPTION_LOGS_KEY, request.url().as_str()),
                async {
                    let response = self.execute(request).await?;
                    if response.status().is_success() {
                        let response = response.text().await?;
                        let response: Vec<SubscriptionLog> =
                            jsonpath_lib::select_as(&response, &self.config().get_subscription_logs_api.json_path)?
                                .remove(0);
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

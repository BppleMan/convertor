use crate::op::OpItem;
use crate::service::service_config::ServiceConfig;
use crate::service::subscription_log::SubscriptionLog;
use color_eyre::eyre::{eyre, WrapErr};
use moka::future::Cache;
use reqwest::{Method, Response, Url};
use serde::Serialize;

pub const CACHED_AUTH_TOKEN_KEY: &str = "CACHED_AUTH_TOKEN";
pub const CACHED_PROFILE_KEY: &str = "CACHED_PROFILE";
pub const CACHED_SUBSCRIPTION_LOGS_KEY: &str = "CACHED_SUBSCRIPTION_LOGS";

pub trait ServiceApi {
    fn config(&self) -> &ServiceConfig;

    fn client(&self) -> &reqwest::Client;

    fn cached_auth_token(&self) -> &Cache<String, String>;

    fn cached_profile(&self) -> &Cache<String, String>;

    fn cached_subscription_logs(&self) -> &Cache<String, Vec<SubscriptionLog>>;

    async fn get_credential(&self) -> color_eyre::Result<OpItem>;

    async fn request<T: Serialize + ?Sized>(
        &self,
        method: Method,
        url: impl AsRef<str>,
        headers: Vec<(impl AsRef<str>, impl AsRef<str>)>,
        form: Option<&T>,
    ) -> color_eyre::Result<Response> {
        let mut request_builder =
            self.client().request(method, url.as_ref()).header(
                "User-Agent",
                concat!("convertor/", env!("CARGO_PKG_VERSION")),
            );
        for (k, v) in headers {
            request_builder = request_builder.header(k.as_ref(), v.as_ref());
        }
        request_builder = if let Some(form) = form {
            request_builder.form(form)
        } else {
            request_builder
        };
        Ok(request_builder.send().await?)
    }

    async fn login(&self) -> color_eyre::Result<String> {
        self.cached_auth_token()
            .try_get_with(CACHED_AUTH_TOKEN_KEY.to_string(), async {
                let login_url = format!(
                    "{}{}{}",
                    self.config().base_url,
                    self.config().prefix_path,
                    self.config().login_api.api_path
                );
                let identity = self.get_credential().await?;
                let response = self
                    .request(
                        Method::POST,
                        &login_url,
                        Vec::<(&str, &str)>::new(),
                        Some(&[
                            ("email", &identity.username),
                            ("password", &identity.password),
                        ]),
                    )
                    .await?;
                if response.status().is_success() {
                    let json_response = response.text().await?;
                    let auth_token = jsonpath_lib::select_as(
                        &json_response,
                        self.config().login_api.json_path,
                    )
                    .wrap_err_with(|| {
                        format!(
                            "failed to select json_path: {}",
                            self.config().login_api.json_path
                        )
                    })?
                    .remove(0);
                    Ok(auth_token)
                } else {
                    Err(eyre!("Login failed: {}", response.status()))
                }
            })
            .await
            .map_err(|e| eyre!(e))
    }

    async fn get_raw_profile(
        &self,
        url: impl AsRef<str>,
    ) -> color_eyre::Result<String> {
        self.cached_profile()
            .try_get_with(
                format!("{}_{}", CACHED_PROFILE_KEY, url.as_ref()),
                async {
                    let response = self
                        .request(
                            Method::GET,
                            url,
                            Vec::<(&str, &str)>::new(),
                            Option::<&str>::None,
                        )
                        .await?;
                    if response.status().is_success() {
                        response.text().await.map_err(Into::into)
                    } else {
                        Err(eyre!(
                            "Get raw profile failed: {}",
                            response.status()
                        ))
                    }
                },
            )
            .await
            .map_err(|e| eyre!(e))
    }

    async fn reset_subscription_url(
        &self,
        auth_token: impl AsRef<str>,
    ) -> color_eyre::Result<Url> {
        let reset_url = format!(
            "{}{}{}",
            self.config().base_url,
            self.config().prefix_path,
            self.config().reset_subscription_url.api_path
        );
        let response = self
            .request(
                Method::POST,
                &reset_url,
                vec![("Authorization", auth_token)],
                Option::<&str>::None,
            )
            .await?;
        if response.status().is_success() {
            let json_response = response.text().await?;
            let url_str: String = jsonpath_lib::select_as(
                &json_response,
                self.config().reset_subscription_url.json_path,
            )
            .wrap_err_with(|| {
                format!(
                    "failed to select json_path: {}",
                    self.config().reset_subscription_url.json_path
                )
            })?
            .remove(0);
            Url::parse(&url_str).map_err(|e| e.into())
        } else {
            Err(eyre!(
                "Reset subscription URL failed: {}",
                response.status()
            ))
        }
    }

    async fn get_subscription_url(
        &self,
        auth_token: impl AsRef<str>,
    ) -> color_eyre::Result<Url> {
        let get_url = format!(
            "{}{}{}",
            self.config().base_url,
            self.config().prefix_path,
            self.config().get_subscription_url.api_path
        );
        let response = self
            .request(
                Method::GET,
                &get_url,
                vec![("authorization", auth_token)],
                Option::<&str>::None,
            )
            .await?;
        if response.status().is_success() {
            let json_response = response.text().await?;
            let url_str: String = jsonpath_lib::select_as(
                &json_response,
                self.config().get_subscription_url.json_path,
            )
            .wrap_err_with(|| {
                format!(
                    "failed to select json_path: {}",
                    self.config().get_subscription_url.json_path
                )
            })?
            .remove(0);
            Url::parse(&url_str).map_err(|e| e.into())
        } else {
            Err(eyre!("Get subscription URL failed: {}", response.status()))
        }
    }

    async fn get_subscription_log(
        &self,
        auth_token: impl AsRef<str>,
    ) -> color_eyre::Result<Vec<SubscriptionLog>> {
        println!("{}_{}", CACHED_SUBSCRIPTION_LOGS_KEY, auth_token.as_ref());
        self.cached_subscription_logs()
            .try_get_with(
                format!(
                    "{}_{}",
                    CACHED_SUBSCRIPTION_LOGS_KEY,
                    auth_token.as_ref()
                ),
                async {
                    let log_url = format!(
                        "{}{}{}",
                        self.config().base_url,
                        self.config().prefix_path,
                        self.config().get_subscription_log.api_path
                    );
                    let response = self
                        .request(
                            Method::GET,
                            &log_url,
                            vec![("Authorization", auth_token)],
                            Option::<&str>::None,
                        )
                        .await?;
                    if response.status().is_success() {
                        let response = response.text().await?;
                        let response: Vec<SubscriptionLog> =
                            jsonpath_lib::select_as(
                                &response,
                                self.config().get_subscription_log.json_path,
                            )?
                            .remove(0);
                        Ok(response)
                    } else {
                        Err(eyre!(
                            "Get subscription log failed: {}",
                            response.status()
                        ))
                    }
                },
            )
            .await
            .map_err(|e| eyre!(e))
    }
}

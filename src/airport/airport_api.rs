use crate::airport::airport_config::AirportConfig;
use crate::op;
use color_eyre::eyre::WrapErr;
use reqwest::{Method, Response, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait AirportApi {
    fn config(&self) -> &AirportConfig;

    fn client(&self) -> &reqwest::Client;

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
        let login_url = format!(
            "{}{}{}",
            self.config().base_url,
            self.config().prefix_path,
            self.config().login_api.api_path
        );
        println!("{}", login_url);
        let identity = op::get_item(self.config().one_password_key).await?;
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
            Err(color_eyre::eyre::eyre!(
                "Login failed: {}",
                response.status()
            ))
        }
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
            Err(color_eyre::eyre::eyre!(
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
            Err(color_eyre::eyre::eyre!(
                "Get subscription URL failed: {}",
                response.status()
            ))
        }
    }

    async fn get_subscription_log<T: DeserializeOwned>(
        &self,
        auth_token: impl AsRef<str>,
    ) -> color_eyre::Result<T> {
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
            let response: T = jsonpath_lib::select_as(
                &response,
                self.config().get_subscription_log.json_path,
            )?
            .remove(0);
            Ok(response)
        } else {
            Err(color_eyre::eyre::eyre!(
                "Get subscription log failed: {}",
                response.status()
            ))
        }
    }
}

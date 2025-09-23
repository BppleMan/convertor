use crate::common::cache::Cache;
use crate::config::subscription_config::Headers;
use crate::error::{ApiFailed, ProviderError};
use redis::aio::ConnectionManager;
use reqwest::Method;
use std::ops::Deref;
use std::time::Duration;
use url::Url;

#[derive(Clone)]
pub struct SubscriptionProvider {
    pub client: reqwest::Client,
    pub cache: Cache<String, String>,
}

impl SubscriptionProvider {
    pub fn new(redis: Option<ConnectionManager>) -> Self {
        let client = reqwest::Client::new();
        let cache = Cache::new(
            redis,
            10,
            Duration::from_secs(60 * 60),
            Duration::from_secs(60 * 60 * 12),
        );
        Self { client, cache }
    }

    pub async fn get_raw_profile(&self, sub_url: Url, headers: Headers) -> Result<String, ProviderError> {
        let mut request_builder = self.client.request(Method::GET, sub_url.clone());
        for (k, v) in headers.deref() {
            request_builder = request_builder.header(k, v);
        }
        let request = request_builder.build().map_err(|e| ProviderError::RequestError {
            reason: "无法构建请求".to_string(),
            source: Box::new(e),
        })?;
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| ProviderError::RequestError {
                reason: "请求失败".to_string(),
                source: Box::new(e),
            })?;
        let response_status = response.status();
        let response_headers = response
            .headers()
            .into_iter()
            .flat_map(|(k, v)| {
                let k = k.to_string();
                v.to_str().map(|v| (k, v.to_string())).ok()
            })
            .collect::<Headers>();
        let response_body = response.text().await.map_err(|e| ProviderError::RequestError {
            reason: "读取响应体失败".to_string(),
            source: Box::new(e),
        })?;
        if response_status.is_success() {
            Ok(response_body)
        } else {
            Err(ProviderError::ApiFailed(Box::new(ApiFailed {
                request_url: sub_url,
                request_method: Method::GET,
                request_headers: headers,
                response_status,
                response_headers,
                response_body,
            })))
        }
    }
}

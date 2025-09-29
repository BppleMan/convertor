use crate::common::cache::{Cache, CacheKey};
use crate::config::subscription_config::Headers;
use crate::error::{ApiFailed, ProviderError, RequestInfo, ResponseInfo};
use redis::aio::ConnectionManager;
use reqwest::Method;
use std::ops::Deref;
use std::time::{Duration, Instant};
use tracing::{debug, instrument};
use url::Url;

#[derive(Clone)]
pub struct SubsProvider {
    pub client: reqwest::Client,
    pub cache: Cache<String, String>,
    pub cache_prefix: String,
}

impl SubsProvider {
    pub fn new(redis: Option<ConnectionManager>, cache_prefix: Option<impl AsRef<str>>) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(5000))
            // .connection_verbose(true)
            .build()
            .expect("构建 reqwest 客户端失败");
        let cache = Cache::new(
            redis,
            10,
            #[cfg(debug_assertions)]
            Duration::from_secs(60 * 60 * 24),
            #[cfg(not(debug_assertions))]
            Duration::from_secs(60 * 60),
            Duration::from_secs(60 * 60 * 12),
        );
        let cache_prefix = cache_prefix
            .as_ref()
            .map(|s| s.as_ref().to_string())
            .unwrap_or_else(|| "convertor:".to_string());
        Self {
            client,
            cache,
            cache_prefix,
        }
    }

    pub async fn get_raw_profile(&self, sub_url: Url, headers: Headers) -> Result<String, ProviderError> {
        let cache_key = CacheKey::new(&self.cache_prefix, sub_url.to_string(), None);
        let raw_profile = self
            .cache
            .try_get_with(cache_key, async { self.fetch(sub_url, headers).await })
            .await?;
        Ok(raw_profile)
    }

    #[instrument(skip(self))]
    pub async fn fetch(&self, sub_url: Url, headers: Headers) -> Result<String, ProviderError> {
        let mut request_info = RequestInfo::new(sub_url.clone(), Method::GET);

        let mut rb = self.client.request(Method::GET, sub_url.clone());
        for (k, v) in headers.deref() {
            rb = rb.header(k, v);
        }
        let req = rb.build().map_err(|e| ProviderError::RequestError {
            reason: "无法构建请求".to_string(),
            source: Box::new(e),
            request_info: request_info.clone(),
        })?;

        // Request 侧字段
        request_info = request_info.patch(&req);
        let request_body_len = req.body().and_then(|b| b.as_bytes()).map(|b| b.len()).unwrap_or(0);

        let started = Instant::now();

        // —— 发出请求
        let resp = self
            .client
            .execute(req)
            .await
            .map_err(|e| ProviderError::RequestError {
                reason: "请求失败".to_string(),
                source: Box::new(e),
                request_info: request_info.clone(),
            })?;

        let elapsed_headers_ms = started.elapsed().as_millis();

        // 响应头字段
        let response_info = ResponseInfo {
            final_url: resp.url().clone(),
            status: resp.status(),
            status_text: resp.status().canonical_reason().map(|s| s.to_string()),
            version: resp.version(),
            headers: Headers::from_header_map(resp.headers().clone()),
            body: None,
        };

        // 读取完整 body
        let response_body_text = resp.text().await.map_err(|e| ProviderError::ResponseError {
            reason: "读取响应体失败".to_string(),
            source: Box::new(e),
            response_info: response_info.clone(),
        })?;

        let response_body_len = response_body_text.len();
        let elapsed_total_ms = started.elapsed().as_millis();
        let bytes_out = request_body_len as u64;
        let bytes_in = response_body_len as u64;

        // 统一打印完整请求日志
        debug!(
            target: "httpv",
            req_id = %request_info.req_id,
            method = %request_info.method,
            url = %request_info.url,
            final_url = %response_info.final_url,
            status = response_info.status.as_u16(),
            ttfb_ms = elapsed_headers_ms,
            total_ms = elapsed_total_ms,
            bytes_out = bytes_out,
            bytes_in = bytes_in,
            "HTTP request completed"
        );

        if response_info.status.is_success() {
            Ok(response_body_text)
        } else {
            // 与你原有错误结构对齐
            Err(ProviderError::ApiFailed(Box::new(ApiFailed {
                request: request_info,
                response: response_info,
            })))
        }
    }
}

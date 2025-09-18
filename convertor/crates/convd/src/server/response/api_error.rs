use axum::http::header::ToStrError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use convertor::config::client_config::ProxyClient;
use convertor::provider_api::ProviderApiError;
use convertor::url::url_error::{QueryError, UrlBuilderError};
use redis::RedisError;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("获取原始非转换配置失败, 遇到错误的客户端: {0}")]
    RawProfileUnsupportedClient(ProxyClient),

    #[error("没有找到对应的订阅提供者")]
    NoSubProvider,

    #[error(transparent)]
    ConvertorUrl(#[from] UrlBuilderError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    QueryError(#[from] QueryError),

    #[error(transparent)]
    ProviderApiError(#[from] ProviderApiError),

    #[error(transparent)]
    ToStr(#[from] ToStrError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    CacheError(#[from] Arc<ApiError>),

    #[error("Redis 错误: {0:?}")]
    RedisError(#[from] RedisError),

    #[error(transparent)]
    Unknown(#[from] color_eyre::Report),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = format!("{self:?}");
        let message = console::strip_ansi_codes(&message).to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}

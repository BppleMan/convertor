use axum::http::StatusCode;
use axum::http::header::ToStrError;
use axum::response::{IntoResponse, Response};
use convertor::config::proxy_client::ProxyClient;
use convertor::error::{ProviderError, QueryError, UrlBuilderError};
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
    UrlBuilderError(#[from] UrlBuilderError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    QueryError(#[from] QueryError),

    #[error(transparent)]
    ProviderError(#[from] ProviderError),

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
        let message = format!("{self}");
        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}

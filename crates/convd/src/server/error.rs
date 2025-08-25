use axum::http::StatusCode;
use axum::http::header::ToStrError;
use axum::response::{IntoResponse, Response};
use convertor::common::config::proxy_client_config::ProxyClient;
use convertor::url::url_error::{QueryError, UrlBuilderError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("获取原始非转换配置失败, 遇到错误的客户端: {0}")]
    RawProfileUnsupportedClient(ProxyClient),

    #[error("没有找到对应的订阅提供者")]
    NoSubProvider,

    // #[error("没有找到对应的代理客户端")]
    // NoProxyClient,
    #[error(transparent)]
    ConvertorUrl(#[from] UrlBuilderError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    QueryError(#[from] QueryError),

    #[error(transparent)]
    Eyre(#[from] color_eyre::Report),

    #[error(transparent)]
    ToStr(#[from] ToStrError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    CacheError(#[from] Arc<AppError>),
    // #[error("读写锁异常: {0}")]
    // RwLockError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = format!("{self:?}");
        let message = console::strip_ansi_codes(&message).to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}

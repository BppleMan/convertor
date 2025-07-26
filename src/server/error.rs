use crate::core::url_builder::ConvertorUrlError;
use axum::http::StatusCode;
use axum::http::header::ToStrError;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("没有找到对应的订阅提供者")]
    NoSubProvider,

    #[error("没有找到对应的代理客户端")]
    NoProxyClient,

    #[error(transparent)]
    ConvertorUrl(#[from] ConvertorUrlError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    Eyre(#[from] color_eyre::Report),

    #[error(transparent)]
    ToStr(#[from] ToStrError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    CacheError(#[from] Arc<AppError>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = format!("{self:?}");
        let message = console::strip_ansi_codes(&message).to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}

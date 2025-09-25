use crate::server::response::{ApiResponse, ApiStatus};
use axum::http::header::ToStrError;
use convertor::config::proxy_client::ProxyClient;
use convertor::error::{ParseError, ProviderError, QueryError, RenderError, UrlBuilderError};
use redis::RedisError;
use std::sync::Arc;
use thiserror::Error;

macro_rules! define_error {
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$var_attr:meta])*
                $var:ident
                $( ( $( $(#[$fattr:meta])* $fty:ty ),+ $(,)? ) )?
                $( { $( $(#[$field_attr:meta])* $fname:ident : $ftyt:ty ),+ $(,)? } )?
                ; $vi:expr
                $(,)?
            )*
        }
    ) => {
        $(#[$enum_attr])*
        $vis enum $name {
            $(
                $(#[$var_attr])*
                $var
                $( ( $( $(#[$fattr])* $fty ),+ ) )?
                $( { $( $(#[$field_attr])* $fname : $ftyt ),+ } )?
            ),*
        }

        impl From<$name> for ApiResponse<()> {
            fn from(value: $name) -> Self {
                ApiResponse::error_with_status(value.into())
            }
        }

        impl From<AppError> for ApiStatus {
            fn from(value: AppError) -> Self {
                match value {
                    $($name::$var(e) => Self::new($vi, e),)*
                }
            }
        }

    };
}

define_error! {
    /// 用于表示应用程序中的各种错误类型。
    #[allow(clippy::large_enum_variant)]
    #[derive(Debug, Error)]
    pub enum AppError {
        #[error("获取订阅商原始配置失败, 遇到错误的客户端: {0}")]
        RawProfileUnsupportedClient(ProxyClient) ; 1001,

        #[error("Unauthorized: {0}")]
        Unauthorized(String) ; 1002,

        #[error(transparent)]
        UrlBuilderError(#[from] UrlBuilderError) ; 1003,

        #[error(transparent)]
        QueryError(#[from] QueryError) ; 1004,

        #[error(transparent)]
        ProviderError(#[from] ProviderError) ; 1005,

        #[error(transparent)]
        ParseError(#[from] ParseError) ; 1006,

        #[error(transparent)]
        RenderError(#[from] RenderError) ; 1007,

        #[error(transparent)]
        ToStr(#[from] ToStrError) ; 1008,

        #[error(transparent)]
        Utf8Error(#[from] std::str::Utf8Error) ; 1009,

        #[error(transparent)]
        CacheError(#[from] Arc<AppError>) ; 1010,

        #[error("Redis 错误: {0:?}")]
        RedisError(#[from] RedisError) ; 1011,

        #[error(transparent)]
        JsonError(#[from] serde_json::Error) ; 1012,
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("获取订阅商原始配置失败, 遇到错误的客户端: {0}")]
    RawProfileUnsupportedClient(ProxyClient),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    UrlBuilderError(#[from] UrlBuilderError),

    #[error(transparent)]
    QueryError(#[from] QueryError),

    #[error(transparent)]
    ProviderError(#[from] ProviderError),

    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error(transparent)]
    RenderError(#[from] RenderError),

    #[error(transparent)]
    ToStr(#[from] ToStrError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    CacheError(#[from] Arc<SnapshotError>),

    #[error("Redis 错误: {0:?}")]
    RedisError(#[from] RedisError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

// impl From<AppError> for ApiResponse<()> {
//     fn from(value: AppError) -> Self {
//         ApiResponse::error_with_status(value.into())
//     }
// }

// impl From<AppError> for ApiStatus {
//     fn from(value: AppError) -> Self {
//         match value {
//             AppError::RawProfileUnsupportedClient(e) => Self::new(1001, e),
//             AppError::UrlBuilderError(e) => Self::new(1002, e),
//             AppError::Unauthorized(e) => Self::new(1003, e),
//             AppError::QueryError(e) => Self::new(1005, e),
//             AppError::ProviderError(e) => Self::new(1006, e),
//             AppError::ParseError(e) => Self::new(9, e),
//             AppError::RenderError(e) => Self::new(1004, e),
//             AppError::ToStr(e) => Self::new(1007, e),
//             AppError::Utf8Error(e) => Self::new(1008, e),
//             AppError::CacheError(e) => Self::new(1009, e),
//             AppError::RedisError(e) => Self::new(10010, e),
//             AppError::JsonError(e) => Self::new(1011, e),
//         }
//     }
// }

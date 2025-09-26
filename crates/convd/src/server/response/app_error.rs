use crate::server::response::ApiResponse;
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
                match value {
                    $($name::$var(e) => Self::from_error(stringify!($var), e),)*
                }
            }
        }

        // impl From<$name> for ApiStatus {
        //     fn from(value: $name) -> Self {
        //         match value {
        //             $($name::$var(e) => Self::from_error($vi, e),)*
        //         }
        //     }
        // }

    };
}

define_error! {
    /// 用于表示应用程序中的各种错误类型。
    #[allow(clippy::large_enum_variant)]
    #[derive(Debug, Error)]
    pub enum AppError {
        #[error(transparent)]
        RequestError(#[from] RequestError),

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
        ToStrError(#[from] ToStrError),

        #[error(transparent)]
        Utf8Error(#[from] std::str::Utf8Error),

        #[error(transparent)]
        CacheError(#[from] Arc<AppError>),

        #[error("Redis 错误: {0:?}")]
        RedisError(#[from] RedisError),

        #[error(transparent)]
        JsonError(#[from] serde_json::Error),
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

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("获取订阅商原始配置失败, 遇到错误的客户端: {0}")]
    UnsupportedClient(ProxyClient),

    #[error("请求失败, 未找到有效的 secret 令牌: {0}")]
    Unauthorized(String),
}

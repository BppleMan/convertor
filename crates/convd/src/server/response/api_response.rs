use crate::server::response::AppError;
use crate::server::response::api_error::ApiError;
use crate::server::response::api_status::ApiStatus;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: serde::Serialize,
{
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            status: ApiStatus::OK,
            data: Some(data),
        }
    }

    pub fn ok_with_message(data: Option<T>, message: impl Display) -> Self {
        Self {
            status: ApiStatus::OK.with_message(message),
            data,
        }
    }
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    pub fn error() -> Self {
        Self {
            status: ApiStatus::ERROR,
            data: None,
        }
    }

    pub fn error_with_status(status: ApiStatus) -> Self {
        Self { status, data: None }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf = BytesMut::with_capacity(256).writer();
        match serde_json::to_writer(&mut buf, &self) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => {
                let api_error = ApiError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    error: AppError::JsonError(err),
                };
                api_error.into_response()
            }
        }
    }
}

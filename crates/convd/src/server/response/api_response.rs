use crate::server::response::api_error::ApiError;
use crate::server::response::{AppError, RequestSnapshot};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Display;
use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: serde::Serialize,
{
    pub status: String,
    pub messages: Vec<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            messages: vec![],
            request: None,
            data: Some(data),
        }
    }

    pub fn with_message(mut self, message: impl Display) -> Self {
        self.messages = vec![Cow::Owned(message.to_string())];
        self
    }

    pub fn with_request(mut self, request: RequestSnapshot) -> Self {
        self.request = Some(request);
        self
    }
}

impl ApiResponse<()> {
    pub fn from_error(status: impl Display, error: impl core::error::Error) -> Self {
        let status = status.to_string();
        let mut messages = vec![Cow::Owned(error.to_string())];
        let mut source = error.source();
        while let Some(src) = source {
            messages.push(Cow::Owned(src.to_string()));
            source = src.source();
        }
        Self {
            status,
            messages,
            request: None,
            data: None::<()>,
        }
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
                    request: None,
                };
                api_error.into_response()
            }
        }
    }
}

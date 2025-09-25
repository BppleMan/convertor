use crate::server::response::{ApiResponse, AppError};
use axum::response::{IntoResponse, Response};
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub struct ApiError {
    pub status: axum::http::StatusCode,
    pub error: AppError,
}

impl ApiError {
    pub fn bad_request(error: impl Into<AppError>) -> Self {
        Self {
            status: axum::http::StatusCode::BAD_REQUEST,
            error: error.into(),
        }
    }

    pub fn internal_server_error(error: impl Into<AppError>) -> Self {
        Self {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error: error.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut buf = BytesMut::with_capacity(256).writer();
        let api_response: ApiResponse<()> = self.error.into();
        match serde_json::to_writer(&mut buf, &api_response) {
            Ok(()) => (
                self.status,
                [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                [(axum::http::header::CONTENT_TYPE, mime::TEXT_PLAIN_UTF_8.as_ref())],
                Bytes::from(format!("Failed to serialize error response: {}", err)),
            )
                .into_response(),
        }
    }
}

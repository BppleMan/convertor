use crate::server::response::{ApiResponse, AppError, RequestSnapshot};
use axum::response::{IntoResponse, Response};
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub struct ApiError {
    pub status: axum::http::StatusCode,
    pub error: AppError,
    pub request: Option<RequestSnapshot>,
}

impl ApiError {
    pub fn bad_request(error: impl Into<AppError>) -> Self {
        Self {
            status: axum::http::StatusCode::BAD_REQUEST,
            error: error.into(),
            request: None,
        }
    }

    pub fn internal_server_error(error: impl Into<AppError>) -> Self {
        Self {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error: error.into(),
            request: None,
        }
    }

    pub fn with_request(mut self, request: RequestSnapshot) -> Self {
        self.request = Some(request);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(mut self) -> Response {
        let mut buf = BytesMut::with_capacity(256).writer();
        let request = self.request.take();
        let mut api_response: ApiResponse<()> = self.error.into();
        if let Some(request) = request {
            api_response = api_response.with_request(request);
        }
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

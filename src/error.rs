use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct AppError(#[from] pub anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = format!("{:?}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}

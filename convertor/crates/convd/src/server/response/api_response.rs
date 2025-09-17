use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: serde::Serialize,
{
    pub status: ApiStatus,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: serde::Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            status: ApiStatus::Ok,
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    pub fn ok_with_message(data: Option<T>, message: impl AsRef<str>) -> Self {
        Self {
            status: ApiStatus::Ok,
            message: message.as_ref().to_string(),
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
            status: ApiStatus::Error,
            message: "error".to_string(),
            data: None,
        }
    }

    pub fn error_with_message(message: impl AsRef<str>) -> Self {
        Self {
            status: ApiStatus::Error,
            message: message.as_ref().to_string(),
            data: None,
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum ApiStatus {
    #[default]
    Ok = 0,
    Error = -1,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // Extracted into separate fn so it's only compiled once for all T.
        fn make_response(buf: BytesMut, ser_result: serde_json::Result<()>) -> Response {
            match ser_result {
                Ok(()) => (
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                    )],
                    buf.freeze(),
                )
                    .into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                    )],
                    err.to_string(),
                )
                    .into_response(),
            }
        }

        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        let res = serde_json::to_writer(&mut buf, &self);
        make_response(buf.into_inner(), res)
    }
}

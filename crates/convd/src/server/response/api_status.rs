use crate::server::response::ApiResponse;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiStatus {
    pub code: i64,
    pub message: Cow<'static, str>,
}

impl ApiStatus {
    pub const OK: Self = ApiStatus {
        code: 0,
        message: Cow::Borrowed("ok"),
    };

    pub const ERROR: Self = ApiStatus {
        code: 1,
        message: Cow::Borrowed("error"),
    };
}

impl ApiStatus {
    pub fn with_code(mut self, code: i64) -> Self {
        self.code = code;
        self
    }

    pub fn with_message(mut self, message: impl AsRef<str>) -> Self {
        self.message = Cow::Owned(message.as_ref().to_string());
        self
    }
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

use crate::server::response::ApiResponse;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Display;
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
        message: Cow::Borrowed("unknown error"),
    };
}

impl ApiStatus {
    pub fn new(code: i64, message: impl Display) -> Self {
        Self {
            code,
            message: Cow::Owned(message.to_string()),
        }
    }

    pub fn with_code(mut self, code: i64) -> Self {
        self.code = code;
        self
    }

    pub fn with_message(mut self, message: impl Display) -> Self {
        self.message = Cow::Owned(message.to_string());
        self
    }
}

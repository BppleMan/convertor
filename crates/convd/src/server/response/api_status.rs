use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Display;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiStatus {
    pub code: i64,
    pub message: Vec<Cow<'static, str>>,
}

impl ApiStatus {
    pub fn ok() -> Self {
        ApiStatus {
            code: 0,
            message: vec![Cow::Borrowed("ok")],
        }
    }
}

impl ApiStatus {
    pub fn new(code: i64, message: impl Display) -> Self {
        Self {
            code,
            message: vec![Cow::Owned(message.to_string())],
        }
    }

    pub fn from_error(code: i64, error: impl core::error::Error) -> Self {
        let mut message = vec![Cow::Owned(error.to_string())];
        let mut source = error.source();
        while let Some(src) = source {
            message.push(Cow::Owned(src.to_string()));
            source = src.source();
        }
        Self { code, message }
    }

    pub fn with_code(mut self, code: i64) -> Self {
        self.code = code;
        self
    }

    pub fn with_message(mut self, message: impl Display) -> Self {
        self.message = vec![Cow::Owned(message.to_string())];
        self
    }
}

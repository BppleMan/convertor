use crate::server::response::api_status::ApiStatus;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T>
where
    T: serde::Serialize,
{
    pub status: ApiStatus,
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

    pub fn ok_with_message(data: Option<T>, message: impl AsRef<str>) -> Self {
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

    pub fn error_with_message(message: impl AsRef<str>) -> Self {
        Self {
            status: ApiStatus::ERROR.with_message(message),
            data: None,
        }
    }
}

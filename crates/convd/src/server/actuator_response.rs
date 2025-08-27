use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorResponse<T> {
    pub status: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> Default for ActuatorResponse<T>
where
    T: Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self {
            status: ActuatorResponseStatus::OK,
            message: "OK".to_string(),
            data: None,
        }
    }
}

impl<T> ActuatorResponse<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn ok() -> ActuatorResponse<T> {
        Self {
            status: 0,
            message: "OK".to_string(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn ok_data(data: T) -> Self {
        Self {
            status: 200,
            message: "OK".to_string(),
            data: Some(data),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub enum ActuatorResponseStatus {
    #[default]
    OK = 0,
}

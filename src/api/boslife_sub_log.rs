use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BosLifeSubLog {
    pub user_id: u64,
    pub ip: String,
    pub location: String,
    pub isp: String,
    pub host: String,
    pub ua: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BosLifeSubLogs(pub Vec<BosLifeSubLog>);

impl Display for BosLifeSubLogs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = serde_json::to_string_pretty(self).expect("JSON serialization failed");
        write!(f, "{}", string)
    }
}

impl From<String> for BosLifeSubLogs {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).expect("JSON deserialization failed")
    }
}

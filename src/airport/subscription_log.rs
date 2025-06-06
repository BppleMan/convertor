use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionLog {
    pub user_id: u64,
    pub ip: String,
    pub location: String,
    pub isp: String,
    pub host: String,
    pub ua: String,
    pub created_at: u64,
}

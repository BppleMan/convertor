use crate::config::convertor_config::Credential;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BosLifeCredential {
    pub email: String,
    pub password: String,
}

impl Credential for BosLifeCredential {
    fn get_username(&self) -> &str {
        &self.email
    }

    fn get_password(&self) -> &str {
        &self.password
    }
}

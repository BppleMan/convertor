use convertor::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod client_config;

pub use client_config::*;
use convertor::config::proxy_client::ProxyClient;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ConflyConfig {
    #[serde(flatten)]
    pub common: Config,

    #[serde(flatten)]
    pub clients: HashMap<ProxyClient, ClientConfig>,
}

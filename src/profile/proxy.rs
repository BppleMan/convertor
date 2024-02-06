use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    pub scheme: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub udp: Option<bool>,
    pub sni: Option<String>,
    pub skip_cert_verify: Option<bool>,
    pub tfo: Option<bool>,
}

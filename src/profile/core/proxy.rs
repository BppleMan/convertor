use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Proxy {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub udp: Option<bool>,
    pub tfo: Option<bool>,
    pub cipher: Option<String>,
    pub sni: Option<String>,
    #[serde(rename = "skip-cert-verify", default)]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip)]
    pub comment: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub udp: bool,
    pub cipher: Option<String>,
    pub sni: Option<String>,
    #[serde(rename = "skip-cert-verify", default)]
    pub skip_cert_verify: Option<bool>,
}

impl Proxy {
    pub fn serialize(&self) -> String {
        let mut fields = vec![
            format!(r#"name: "{}""#, self.name),
            format!(r#"type: "{}""#, self.r#type),
            format!(r#"server: "{}""#, self.server),
            format!(r#"port: {}"#, self.port),
            format!(r#"password: "{}""#, self.password),
            format!(r#"udp: {}"#, self.udp),
        ];
        if let Some(cipher) = &self.cipher {
            fields.push(format!(r#"cipher: "{}""#, cipher));
        }
        if let Some(sni) = &self.sni {
            fields.push(format!(r#"sni: "{}""#, sni));
        }
        if let Some(skip_cert_verify) = self.skip_cert_verify {
            fields.push(format!(r#"skip-cert-verify: {}"#, skip_cert_verify));
        }
        format!("- {} {} {}", "{", fields.join(", "), "}")
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: ProxyGroupType,
    pub proxies: Vec<String>,
}

impl ProxyGroup {
    pub fn new(name: String, r#type: ProxyGroupType, proxies: Vec<String>) -> Self {
        Self { name, r#type, proxies }
    }

    pub fn serialize(&self) -> String {
        let fields = [
            format!(r#"name: "{}""#, self.name),
            format!(r#"type: "{}""#, self.r#type.serialize()),
            format!(
                r#"proxies: [{}]"#,
                self.proxies
                    .iter()
                    .map(|p| format!(r#""{}""#, p))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ];
        format!("- {} {} {}", "{", fields.join(", "), "}")
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum ProxyGroupType {
    #[serde(rename = "select")]
    Select,
    #[default]
    #[serde(rename = "url-test")]
    UrlTest,
}

impl ProxyGroupType {
    pub fn serialize(&self) -> String {
        match self {
            ProxyGroupType::Select => "select".to_string(),
            ProxyGroupType::UrlTest => "url-test".to_string(),
        }
    }
}

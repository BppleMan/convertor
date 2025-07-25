use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RuleProvider {
    pub r#type: String,
    pub url: String,
    pub path: String,
    pub interval: u64,
    #[serde(rename = "size-limit")]
    pub size_limit: u64,
    pub format: String,
    pub behavior: String,
}

impl RuleProvider {
    pub fn new(url: impl ToString, file_name: impl AsRef<str>, interval: u64) -> Self {
        Self {
            r#type: "http".to_string(),
            url: url.to_string(),
            path: format!("./rule_providers/{}.yaml", file_name.as_ref()),
            interval,
            size_limit: 0,
            format: "yaml".to_string(),
            behavior: "classical".to_string(),
        }
    }

    pub fn serialize(&self) -> String {
        let fields = [
            format!(r#"type: "{}""#, self.r#type),
            format!(r#"url: "{}""#, self.url),
            format!(r#"path: "{}""#, self.path),
            format!(r#"interval: {}"#, self.interval),
            format!(r#"size-limit: {}"#, self.size_limit),
            format!(r#"format: "{}""#, self.format),
            format!(r#"behavior: "{}""#, self.behavior),
        ];
        format!("{} {} {}", "{", fields.join(", "), "}")
    }
}

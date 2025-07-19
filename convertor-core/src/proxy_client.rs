use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ProxyClient {
    #[serde(rename = "surge")]
    Surge,
    #[serde(rename = "clash")]
    Clash,
}

pub struct ProxyClientConfig {
    pub client: ProxyClient,
    pub config_dir: PathBuf,
}

impl ProxyClient {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyClient::Surge => "surge",
            ProxyClient::Clash => "clash",
        }
    }
}

impl FromStr for ProxyClient {
    type Err = ParseClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "surge" => Ok(ProxyClient::Surge),
            "clash" => Ok(ProxyClient::Clash),
            _ => Err(ParseClientError(s.to_string())),
        }
    }
}

impl Display for ProxyClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Error)]
#[error("无法解析客户端: {0}")]
pub struct ParseClientError(String);

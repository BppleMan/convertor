use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[derive(ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyClient {
    #[default]
    Surge,
    Clash,
}

impl ProxyClient {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyClient::Surge => "surge",
            ProxyClient::Clash => "clash",
        }
    }

    pub fn variants() -> &'static [Self] {
        &[Self::Surge, Self::Clash]
    }
}

impl Display for ProxyClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ProxyClient {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "surge" => Ok(ProxyClient::Surge),
            "clash" => Ok(ProxyClient::Clash),
            _ => Err(format!("Invalid proxy client: {}", s)),
        }
    }
}

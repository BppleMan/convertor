use clap::ValueEnum;
use color_eyre::eyre::eyre;
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
pub enum Client {
    #[serde(rename = "surge")]
    Surge,
    #[serde(rename = "clash")]
    Clash,
}

impl Client {
    pub fn as_str(&self) -> &'static str {
        match self {
            Client::Surge => "surge",
            Client::Clash => "clash",
        }
    }
}

impl FromStr for Client {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "surge" => Ok(Client::Surge),
            "clash" => Ok(Client::Clash),
            _ => Err(eyre!("无法识别的客户端类型: {}", s)),
        }
    }
}

impl Display for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

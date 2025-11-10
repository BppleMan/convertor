use clap::ValueEnum;
use clap::builder::PossibleValue;
use std::fmt::Display;

static VARIANTS: &[Registry] = &[Registry::Docker, Registry::Ghcr];

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Registry {
    Docker,
    Ghcr,
    Custom(String),
}

impl Registry {
    pub fn as_url(&self) -> &str {
        match self {
            Registry::Docker => "docker.io",
            Registry::Ghcr => "ghcr.io",
            Registry::Custom(url) => url,
        }
    }
}

impl ValueEnum for Registry {
    fn value_variants<'a>() -> &'a [Self] {
        VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.to_string()).hide(matches!(self, Registry::Custom(_))))
    }
}

impl Display for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Registry::Docker => write!(f, "docker"),
            Registry::Ghcr => write!(f, "ghcr"),
            Registry::Custom(_) => write!(f, "`custom_registry`"),
        }
    }
}

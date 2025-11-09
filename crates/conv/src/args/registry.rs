use clap::ValueEnum;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum Registry {
    Local,
    Docker,
    Ghcr,
    Harbor,
}

impl Registry {
    pub fn as_url(&self) -> &'static str {
        match self {
            Registry::Local => "local",
            Registry::Docker => "docker.io",
            Registry::Ghcr => "ghcr.io",
            Registry::Harbor => "10.0.0.31:30083",
        }
    }

    pub fn as_tag_prefix(&self, name: impl AsRef<str>) -> String {
        match self {
            Registry::Local => format!("{}/{}", self.as_url(), name.as_ref()),
            Registry::Docker => format!("{}/{}", self.as_url(), name.as_ref()),
            Registry::Ghcr => format!("{}/{}", self.as_url(), name.as_ref()),
            Registry::Harbor => format!("{}/{}", self.as_url(), name.as_ref()),
        }
    }

    pub fn env_prefix(&self) -> &'static str {
        match self {
            Registry::Local => "LOCAL",
            Registry::Docker => "DOCKER",
            Registry::Ghcr => "GHCR",
            Registry::Harbor => "HARBOR",
        }
    }
}

impl Display for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Registry::Local => write!(f, "local"),
            Registry::Docker => write!(f, "docker"),
            Registry::Ghcr => write!(f, "ghcr"),
            Registry::Harbor => write!(f, "harbor"),
        }
    }
}

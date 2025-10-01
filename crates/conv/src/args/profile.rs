use clap::ValueEnum;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, ValueEnum)]
pub enum Profile {
    Dev,
    Prod,
}

impl Profile {
    pub fn as_cargo_profile(&self) -> &'static str {
        match self {
            Profile::Dev => "dev",
            Profile::Prod => "release",
        }
    }

    pub fn as_cargo_target_dir(&self) -> &'static str {
        match self {
            Profile::Dev => "debug",
            Profile::Prod => "release",
        }
    }

    pub fn as_dashboard_profile(&self) -> &'static str {
        match self {
            Profile::Dev => "development",
            Profile::Prod => "production",
        }
    }

    pub fn as_image_registry(&self) -> &'static str {
        match self {
            Profile::Dev => "local",
            Profile::Prod => "ghcr.io/bppleman/convertor",
        }
    }

    // pub fn as_image_tag(&self, name: impl AsRef<str>, version: impl AsRef<str>) -> String {
    //     match self {
    //         Profile::Dev => format!("{}/{}:{}", self.registry(), name.as_ref(), version.as_ref());,
    //         Profile::Prod => format!("{}/{}:{}", self.registry(), name.as_ref(), version.as_ref()),
    //     }
    // }
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Dev => write!(f, "dev"),
            Profile::Prod => write!(f, "prod"),
        }
    }
}

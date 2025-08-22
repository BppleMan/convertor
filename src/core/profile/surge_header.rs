use crate::core::url_builder::profile_url::ProfileUrl;
use crate::core::url_builder::raw_url::RawUrl;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SurgeHeader {
    Raw {
        shebang: &'static str,
        url: RawUrl,
        interval: u64,
        strict: bool,
    },
    Profile {
        shebang: &'static str,
        url: ProfileUrl,
        interval: u64,
        strict: bool,
    },
}

pub enum SurgeHeaderType {
    Raw,
    RawProfile,
    Profile,
}

impl SurgeHeader {
    pub fn new_raw(url: RawUrl, interval: u64, strict: bool) -> Self {
        Self::Raw {
            shebang: "#!MANAGED-CONFIG",
            url,
            interval,
            strict,
        }
    }

    pub fn new_convertor(url: ProfileUrl, interval: u64, strict: bool) -> Self {
        Self::Profile {
            shebang: "#!MANAGED-CONFIG",
            url,
            interval,
            strict,
        }
    }
}

impl Display for SurgeHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SurgeHeader::Raw {
                shebang,
                url,
                interval,
                strict,
            } => {
                write!(f, "{} {} interval={} strict={}", shebang, url, interval, strict)
            }
            SurgeHeader::Profile {
                shebang,
                url,
                interval,
                strict,
            } => {
                write!(f, "{} {} interval={} strict={}", shebang, url, interval, strict)
            }
        }
    }
}

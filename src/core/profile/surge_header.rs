use crate::core::url_builder::convertor_url::ConvertorUrl;
use crate::core::url_builder::raw_sub_url::RawSubUrl;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SurgeHeader {
    Raw {
        shebang: &'static str,
        url: RawSubUrl,
        interval: u64,
        strict: bool,
    },
    Convertor {
        shebang: &'static str,
        url: ConvertorUrl,
        interval: u64,
        strict: bool,
    },
}

impl SurgeHeader {
    pub fn new_raw(url: RawSubUrl, interval: u64, strict: bool) -> Self {
        Self::Raw {
            shebang: "#!MANAGED-CONFIG",
            url,
            interval,
            strict,
        }
    }

    pub fn new_convertor(url: ConvertorUrl, interval: u64, strict: bool) -> Self {
        Self::Convertor {
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
            SurgeHeader::Convertor {
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

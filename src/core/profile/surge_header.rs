use crate::core::convertor_url::ConvertorUrl;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub struct SurgeHeader {
    pub shebang: &'static str,
    pub url: ConvertorUrl,
    pub interval: u64,
    pub strict: bool,
}

impl SurgeHeader {
    pub fn new(url: ConvertorUrl, interval: u64, strict: bool) -> Self {
        Self {
            shebang: "#!MANAGED-CONFIG",
            url,
            interval,
            strict,
        }
    }
}

impl Display for SurgeHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} interval={} strict={}",
            self.shebang, self.url, self.interval, self.strict
        )
    }
}

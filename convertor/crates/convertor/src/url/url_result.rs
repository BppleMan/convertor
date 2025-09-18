use crate::url::convertor_url::ConvertorUrl;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlResult {
    pub raw_url: ConvertorUrl,
    pub raw_profile_url: ConvertorUrl,
    pub profile_url: ConvertorUrl,
    pub sub_logs_url: ConvertorUrl,
    pub rule_providers_url: Vec<ConvertorUrl>,
}

impl Display for UrlResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.raw_url.desc)?;
        writeln!(f, "{}", self.raw_url)?;
        writeln!(f, "{}", self.profile_url.desc)?;
        writeln!(f, "{}", self.profile_url)?;
        writeln!(f, "{}", self.raw_profile_url.desc)?;
        writeln!(f, "{}", self.raw_profile_url)?;
        writeln!(f, "{}", self.sub_logs_url.desc)?;
        writeln!(f, "{}", self.sub_logs_url)?;
        for url in &self.rule_providers_url {
            writeln!(f, "{}", url.desc)?;
            writeln!(f, "{url}")?;
        }
        Ok(())
    }
}

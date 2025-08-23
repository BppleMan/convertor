use super::expand_env_vars;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SurgeConfig {
    interval: u64,
    strict: bool,
    surge_dir: String,
    main_profile_name: String,
    raw_profile_name: Option<String>,
    raw_sub_name: Option<String>,
    rules_name: Option<String>,
    sub_logs_name: Option<String>,
}

impl SurgeConfig {
    pub fn template() -> Self {
        Self {
            interval: 43200,
            strict: true,
            surge_dir: "${ICLOUD}/../iCloud~com~nssurge~inc/Documents/surge".to_string(),
            main_profile_name: "surge.conf".to_string(),
            raw_profile_name: Some("raw.conf".to_string()),
            raw_sub_name: Some("BosLife.conf".to_string()),
            rules_name: Some("rules.dconf".to_string()),
            sub_logs_name: Some("subscription_logs.js".to_string()),
        }
    }
}

impl SurgeConfig {
    pub fn interval(&self) -> u64 {
        self.interval
    }

    pub fn strict(&self) -> bool {
        self.strict
    }

    pub fn set_surge_dir(&mut self, surge_dir: String) {
        self.surge_dir = surge_dir;
    }

    pub fn surge_dir(&self) -> PathBuf {
        expand_env_vars(&self.surge_dir).into()
    }

    pub fn main_profile_path(&self) -> PathBuf {
        self.surge_dir().join(expand_env_vars(&self.main_profile_name))
    }

    pub fn raw_profile_path(&self) -> Option<PathBuf> {
        self.raw_profile_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }

    pub fn raw_sub_path(&self) -> Option<PathBuf> {
        self.raw_sub_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }

    pub fn rules_path(&self) -> Option<PathBuf> {
        self.rules_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }

    pub fn sub_logs_path(&self) -> Option<PathBuf> {
        self.sub_logs_name
            .as_ref()
            .map(|name| self.surge_dir().join(expand_env_vars(name)))
    }
}

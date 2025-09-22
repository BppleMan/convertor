use std::path::PathBuf;

use convertor::config::proxy_client::ProxyClient;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static ENV_VAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\$\{([A-Za-z0-9_]+)}"#).expect("Failed to compile environment variable regex"));

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    config_dir: String,
    main_profile_name: String,
    raw_profile_name: Option<String>,
    raw_sub_name: Option<String>,
    rules_name: Option<String>,
    sub_logs_name: Option<String>,
}

impl ClientConfig {
    pub fn surge_template() -> Self {
        Self {
            config_dir: "${ICLOUD}/../iCloud~com~nssurge~inc/Documents/surge".to_string(),
            main_profile_name: "surge.conf".to_string(),
            raw_profile_name: Some("raw.conf".to_string()),
            raw_sub_name: Some("BosLife.conf".to_string()),
            rules_name: Some("rules.dconf".to_string()),
            sub_logs_name: Some("subscription_logs.js".to_string()),
        }
    }

    pub fn clash_template() -> Self {
        Self {
            config_dir: "${HOME}/.config/mihomo".to_string(),
            main_profile_name: "config.yaml".to_string(),
            ..Default::default()
        }
    }
}

impl ClientConfig {
    pub fn set_surge_dir(&mut self, surge_dir: String) {
        self.config_dir = surge_dir;
    }

    pub fn surge_dir(&self) -> PathBuf {
        expand_env_vars(&self.config_dir).into()
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

fn expand_env_vars(value: impl AsRef<str>) -> String {
    let value = value.as_ref();
    ENV_VAR_REGEX
        .replace_all(value, |caps: &Captures| {
            let name = &caps[1];
            match std::env::var(name) {
                Ok(value) => value,
                Err(_) => name.to_string(),
            }
        })
        .to_string()
}

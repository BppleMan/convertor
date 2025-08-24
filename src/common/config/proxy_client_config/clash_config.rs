use super::expand_env_vars;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ClashConfig {
    interval: u64,
    strict: bool,
    clash_dir: String,
    main_sub_name: String,
    install_path: Option<String>,
}

impl ClashConfig {
    pub fn template() -> Self {
        Self {
            clash_dir: "${HOME}/.config/mihomo".to_string(),
            interval: 43200,
            strict: true,
            main_sub_name: "config.yaml".to_string(),
            install_path: Some("/usr/local/bin/mihomo".to_string()),
        }
    }
}

impl ClashConfig {
    pub fn set_clash_dir(&mut self, clash_dir: String) {
        self.clash_dir = clash_dir;
    }

    pub fn clash_dir(&self) -> PathBuf {
        expand_env_vars(&self.clash_dir).into()
    }

    pub fn interval(&self) -> u64 {
        self.interval
    }

    pub fn strict(&self) -> bool {
        self.strict
    }

    pub fn main_sub_path(&self) -> PathBuf {
        self.clash_dir().join(expand_env_vars(&self.main_sub_name))
    }

    pub fn install_path(&self) -> Option<PathBuf> {
        self.install_path.as_ref().map(|path| expand_env_vars(path).into())
    }
}

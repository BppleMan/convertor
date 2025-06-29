use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use tracing::error;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SurgeConfig {
    #[allow(unused)]
    pub surge_dir: PathBuf,
    pub main_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub rules_config_path: PathBuf,
}

impl SurgeConfig {
    pub fn try_new() -> Result<Self> {
        let icloud_env = std::env::var("ICLOUD")?;
        let icloud_path = Path::new(&icloud_env);
        let ns_surge_path = icloud_path
            .parent()
            .ok_or_eyre("not found icloud's parent")?
            .join("iCloud~com~nssurge~inc")
            .join("Documents");
        let main_config_path = ns_surge_path.join("surge").join("surge.conf");
        let default_config_path = ns_surge_path.join("surge").join("BosLife.conf");
        let rules_config_path = ns_surge_path.join("surge").join("rules.dconf");
        Ok(Self {
            surge_dir: ns_surge_path,
            main_config_path,
            default_config_path,
            rules_config_path,
        })
    }

    pub async fn update_surge_config(&self, convertor_url: &UrlBuilder) -> Result<()> {
        // update BosLife.conf subscription
        let manager_config_header = Self::build_managed_config_header(convertor_url.build_subscription_url("surge")?);
        Self::update_conf(&self.default_config_path, &manager_config_header).await?;

        // update surge.conf subscription
        let surge_conf = Self::build_managed_config_header(convertor_url.build_convertor_url("surge")?);
        Self::update_conf(&self.main_config_path, &surge_conf).await?;

        Ok(())
    }

    pub async fn update_surge_rule_set(&self, convertor_url: &UrlBuilder) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.rules_config_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();

        let find_position = |rst: &RuleSetPolicy| {
            lines
                .iter()
                .position(|l| l.contains(rst.name()))
                .ok_or_eyre(format!("rule set {} not found", rst.name()))
        };

        let pos_and_rst = RuleSetPolicy::all()
            .iter()
            .map(|rst| find_position(rst).map(|pos| (pos, rst)))
            .collect::<Vec<_>>();

        for pair in pos_and_rst {
            match pair {
                Ok((pos, rst)) => {
                    lines[pos] = Cow::Owned(rst.rule_set(&convertor_url.build_rule_set_url("surge", rst)?));
                }
                Err(e) => error!("{e}"),
            }
        }
        let content = lines.join("\n");
        tokio::fs::write(&self.rules_config_path, &content).await?;
        Ok(())
    }

    async fn update_conf(config_path: impl AsRef<Path>, sub_url: impl AsRef<str>) -> Result<()> {
        let mut content = tokio::fs::read_to_string(&config_path).await?;
        let mut lines = content.lines().collect::<Vec<_>>();
        lines[0] = sub_url.as_ref();
        content = lines.join("\n");
        tokio::fs::write(&config_path, &content).await?;
        Ok(())
    }

    pub fn build_managed_config_header(url: impl AsRef<str>) -> String {
        format!("#!MANAGED-CONFIG {} interval=259200 strict=true", url.as_ref())
    }
}

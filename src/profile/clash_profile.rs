use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

mod proxy;
mod proxy_group;
mod rule;

use crate::region::REGIONS;
pub use proxy::*;
pub use proxy_group::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashProfile {
    pub port: u16,
    #[serde(rename = "socks-port")]
    pub socks_port: u16,
    #[serde(rename = "redir-port")]
    pub redir_port: u16,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    pub secret: String,
    pub proxies: Vec<Proxy>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}

impl ClashProfile {
    pub fn from_str(s: impl AsRef<str>) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(s.as_ref())
    }

    pub fn organize_proxy_group(&mut self) {
        let boslife_info_group = self.proxies.iter().filter(|p| !p.name.starts_with("[BosLife]")).fold(
            ProxyGroup::new("BosLife Info".to_string(), ProxyGroupType::Select, vec![]),
            |mut acc, proxy| {
                acc.proxies.push(proxy.name.clone());
                acc
            },
        );
        let proxy_groups =
            self.proxies
                .iter()
                .filter(|p| p.name.starts_with("[BosLife]"))
                .fold(IndexMap::new(), |mut acc, group| {
                    if let Some(proxy_group_name) = REGIONS.iter().find(|region| group.name.contains(region.as_str())) {
                        let proxy_group = acc.entry(proxy_group_name.to_string()).or_insert(ProxyGroup::new(
                            proxy_group_name.to_string(),
                            ProxyGroupType::URLTest,
                            vec![],
                        ));
                        proxy_group.proxies.push(group.name.clone());
                    }
                    acc
                });
        let boslife_group = ProxyGroup::new(
            "BosLife".to_string(),
            ProxyGroupType::Select,
            proxy_groups.keys().map(|k| k.to_string()).collect(),
        );
        self.proxy_groups = vec![boslife_group, boslife_info_group];
        self.proxy_groups.extend(proxy_groups.into_values());
    }
}

impl Display for ClashProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}

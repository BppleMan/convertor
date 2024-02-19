use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

mod proxy;
mod proxy_group;
mod rule;

use crate::profile::group_by;
use crate::region::Region;
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
        let mut proxy_names = self
            .proxies
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<_>>();
        let other = proxy_names.split_off(3);
        let boslife_info_group = ProxyGroup::new(
            "BosLife Info".to_string(),
            ProxyGroupType::Select,
            proxy_names,
        );
        let mut proxy_map = group_by(&other);
        let groups =
            group_by(&proxy_map.keys().map(|k| k.as_str()).collect::<Vec<_>>());
        let mut boslife_group = ProxyGroup::new(
            "BosLife".to_string(),
            ProxyGroupType::Select,
            proxy_map.keys().map(|k| k.to_string()).collect(),
        );
        for (group, proxies) in &groups {
            if !proxy_map.contains_key(group) {
                proxy_map.insert(group.clone(), proxies.clone());
            }
        }
        for (group, proxies) in groups {
            if let Some(region) = Region::detect(group) {
                boslife_group.proxies.push(region.cn.clone());
                if !proxy_map.contains_key(&region.cn) {
                    proxy_map.insert(region.cn.clone(), proxies);
                }
            }
        }
        self.proxy_groups = vec![boslife_group, boslife_info_group];
        self.proxy_groups.extend(proxy_map.into_iter().map(
            |(name, policy)| {
                ProxyGroup::new(name, ProxyGroupType::UrlTest, policy)
            },
        ));
        println!("{:#?}", self.proxy_groups);
    }
}

impl Display for ClashProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}

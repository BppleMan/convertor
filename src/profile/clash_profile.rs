use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

mod proxy;
mod proxy_group;
mod rule;

use crate::config::url_builder::UrlBuilder;
use crate::profile::clash_profile::rule::Rule;
use crate::profile::{Profile, RuleSetPolicy};
use crate::region::Region;
pub use proxy::*;
pub use proxy_group::*;

const TEMPLATE_STR: &str = include_str!("../../assets/clash/template.yaml");

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashProfile {
    pub proxies: Vec<Proxy>,
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<Rule>,
}

impl Profile for ClashProfile {
    fn generate_profile(
        url_builder: &UrlBuilder,
    ) -> color_eyre::Result<String> {
        let mut template: serde_yaml::Value =
            serde_yaml::from_str(TEMPLATE_STR)?;
        if let serde_yaml::Value::String(url) =
            &mut template["proxy-providers"]["convertor"]["url"]
        {
            *url = url_builder.build_proxy_provider_url("clash")?.to_string();
        }
        for rsp in RuleSetPolicy::all() {
            if matches!(rsp, RuleSetPolicy::DirectForceRemoteDnsPolicy) {
                continue;
            }
            if let serde_yaml::Value::String(url) =
                &mut template["rule-providers"][rsp.provider_name()]["url"]
            {
                *url =
                    url_builder.build_rule_set_url("clash", rsp)?.to_string();
            }
        }
        Ok(serde_yaml::to_string(&template)?)
    }

    fn replace_header(&mut self, convertor_url: impl AsRef<str>) {
        todo!()
    }

    fn organize_proxy_group(&mut self) {
        if let serde_yaml::Value::Sequence(proxies) =
            &mut self.internal["proxies"]
        {
            let (region_map, infos) = group_by_region(proxies);
            let boslife_group = ProxyGroup::new(
                "BosLife".to_string(),
                ProxyGroupType::Select,
                region_map
                    .keys()
                    .map(|r| format!("{} {}", r.icon, r.cn))
                    .collect(),
            );
            let boslife_info = ProxyGroup::new(
                "BosLife Info".to_string(),
                ProxyGroupType::Select,
                infos
                    .into_iter()
                    .flat_map(|i| {
                        if let serde_yaml::Value::String(name) =
                            &proxies[i]["name"]
                        {
                            Some(name.to_string())
                        } else {
                            println!("无法解析的代理节点: {:?}", proxies[i]);
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            let region_groups = region_map
                .into_iter()
                .map(|(region, indices)| {
                    let proxies = indices
                        .into_iter()
                        .flat_map(|i| {
                            if let serde_yaml::Value::String(name) =
                                &proxies[i]["name"]
                            {
                                Some(name.to_string())
                            } else {
                                println!(
                                    "无法解析的代理节点: {:?}",
                                    proxies[i]
                                );
                                None
                            }
                        })
                        .collect();
                    ProxyGroup::new(
                        format!("{} {}", region.icon, region.cn),
                        ProxyGroupType::UrlTest,
                        proxies,
                    )
                })
                .collect::<Vec<_>>();
            if let serde_yaml::Value::Sequence(proxy_groups) =
                &mut self.internal["proxy-groups"]
            {
                proxy_groups.clear();
                if let Ok(boslife_group) = serde_yaml::to_value(&boslife_group)
                {
                    proxy_groups.push(boslife_group);
                }
                if let Ok(boslife_info) = serde_yaml::to_value(&boslife_info) {
                    proxy_groups.push(boslife_info);
                }
                for group in region_groups {
                    if let Ok(group) = serde_yaml::to_value(&group) {
                        proxy_groups.push(group);
                    }
                }
            }
        }
    }

    fn extract_rule(&self, policies: impl AsRef<str>) -> String {
        todo!()
    }
}

impl ClashProfile {
    // pub fn organize_proxy_group(&mut self) {
    //     let mut proxy_names = self
    //         .proxies
    //         .iter()
    //         .map(|p| p.name.to_string())
    //         .collect::<Vec<_>>();
    //     let other = proxy_names.split_off(3);
    //     let boslife_info_group = ProxyGroup::new(
    //         "BosLife Info".to_string(),
    //         ProxyGroupType::Select,
    //         proxy_names,
    //     );
    //     let mut proxy_map = group_by_region(&other);
    //     let groups = group_by_region(
    //         &proxy_map.keys().map(|k| k.as_str()).collect::<Vec<_>>(),
    //     );
    //     let mut boslife_group = ProxyGroup::new(
    //         "BosLife".to_string(),
    //         ProxyGroupType::Select,
    //         proxy_map.keys().map(|k| k.to_string()).collect(),
    //     );
    //     for (group, proxies) in &groups {
    //         if !proxy_map.contains_key(group) {
    //             proxy_map.insert(group.clone(), proxies.clone());
    //         }
    //     }
    //     for (group, proxies) in groups {
    //         if let Some(region) = Region::detect(group) {
    //             boslife_group.proxies.push(region.cn.clone());
    //             if !proxy_map.contains_key(&region.cn) {
    //                 proxy_map.insert(region.cn.clone(), proxies);
    //             }
    //         }
    //     }
    //     self.proxy_groups = vec![boslife_group, boslife_info_group];
    //     self.proxy_groups.extend(proxy_map.into_iter().map(
    //         |(name, policy)| {
    //             ProxyGroup::new(name, ProxyGroupType::UrlTest, policy)
    //         },
    //     ));
    // }
}

impl FromStr for ClashProfile {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            internal: serde_yaml::from_str(s)?,
        })
    }
}

impl Display for ClashProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}

fn group_by_region(
    proxies: &mut serde_yaml::value::Sequence,
) -> (IndexMap<&'static Region, Vec<usize>>, Vec<usize>) {
    let match_number = Regex::new(r"^\d+$").unwrap();
    proxies.iter().enumerate().fold(
        (IndexMap::new(), Vec::new()),
        |(mut regions, mut infos), (index, proxy)| {
            if let serde_yaml::Value::String(name) = &proxy["name"] {
                let mut parts = name.split(' ').collect::<Vec<_>>();
                parts.retain(|part| !match_number.is_match(part));
                match parts.iter().find_map(Region::detect) {
                    Some(region) => {
                        regions.entry(region).or_default().push(index)
                    }
                    None => infos.push(index),
                }
            }
            (regions, infos)
        },
    )
}

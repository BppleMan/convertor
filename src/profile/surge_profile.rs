use std::fmt::{Display, Formatter};

use crate::config::surge_config::SurgeConfig;
use crate::profile::{group_by_region, split_and_merge_groups};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;

static SECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\[[^\[\]]+]$"#).unwrap());

const MANAGED_CONFIG_HEADER: &str = "MANAGED-CONFIG";

#[derive(Debug)]
pub struct SurgeProfile {
    sections: IndexMap<String, Vec<String>>,
}

impl SurgeProfile {
    pub fn new(content: impl Into<String>) -> Self {
        let mut content = content.into().lines().map(|s| s.to_string()).collect::<Vec<String>>();
        let mut sections = IndexMap::new();

        let mut section = MANAGED_CONFIG_HEADER.to_string();
        let mut cursor = 0;
        while cursor < content.len() {
            let line = &content[cursor];
            if SECTION_REGEX.is_match(line) {
                let new_section = line.to_string();
                sections.insert(
                    std::mem::replace(&mut section, new_section),
                    content.drain(..cursor).collect::<Vec<_>>(),
                );
                cursor = 0;
            }
            cursor += 1;
        }
        sections.insert(section, content);

        Self { sections }
    }

    pub fn parse(&mut self, url: impl AsRef<str>) {
        self.replace_header(url);
        self.organize_proxy_group();
    }

    fn replace_header(&mut self, convertor_url: impl AsRef<str>) {
        let header = &mut self.sections[MANAGED_CONFIG_HEADER];
        header[0] = SurgeConfig::build_managed_config_header(convertor_url);
    }

    fn organize_proxy_group(&mut self) {
        if let Some(proxies) = self.proxy() {
            let proxies = proxies
                .iter()
                .skip(1)
                .filter(|p| !p.is_empty())
                .filter_map(|p| p.split('=').next().map(|name| name.trim()))
                .collect::<Vec<_>>();
            let grouped_proxies = group_by_region(&proxies);
            let (groups, info_proxies) = split_and_merge_groups(grouped_proxies);
            let boslife_group = format!(
                "BosLife = select, {}",
                groups
                    .keys()
                    .map(|r| format!("{} {}", r.icon, r.cn))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let boslife_info = format!("BosLife Info = select, {}", info_proxies.join(", "));
            if let Some(proxy_group) = self.proxy_group_mut() {
                if !proxy_group.is_empty() {
                    proxy_group.truncate(1);
                }
                proxy_group.push(boslife_group);
                proxy_group.push(boslife_info);
                groups.into_iter().for_each(|(region, proxies)| {
                    let name = format!("{} {}", region.icon, region.cn);
                    proxy_group.push(format!("{name} = url-test, {}", proxies.join(", ")));
                });
                proxy_group.push("".to_string());
                proxy_group.push("".to_string());
            }
        }
    }

    pub fn extract_rule(&self, policies: impl AsRef<str>) -> String {
        if let Some(rules) = self.rule() {
            rules
                .iter()
                .filter(|rule| !rule.starts_with("//"))
                .filter(|rule| !rule.starts_with('#'))
                .filter(|rule| !rule.starts_with("GEOIP,CN"))
                .filter(|rule| !rule.starts_with("FINAL,DIRECT"))
                .filter(|rule| rule.contains(policies.as_ref()))
                .map(|rule| {
                    let rule_part = rule.split(',').collect::<Vec<_>>();
                    if rule_part.len() > 2 {
                        rule_part[0..2].join(",")
                    } else {
                        rule.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            "".to_string()
        }
    }

    pub fn extract_boslife_subscription(&self) -> String {
        self.rule()
            .map(|rules| {
                let mut begin = false;
                let mut boslife = vec!["// Boslife Subscription".to_string()];
                for line in rules {
                    if line.contains("Subscription") && line.starts_with("//") {
                        begin = true;
                        continue;
                    }
                    if line.starts_with("//") {
                        break;
                    }
                    if begin {
                        let rule_part = line.split(',').collect::<Vec<_>>();
                        if rule_part.len() > 2 {
                            boslife.push(rule_part[0..2].join(","));
                        } else {
                            boslife.push(line.to_string())
                        }
                    }
                }
                boslife.join("\n")
            })
            .unwrap_or_default()
    }

    fn proxy(&self) -> Option<&Vec<String>> {
        self.sections.get("[Proxy]")
    }

    fn proxy_group_mut(&mut self) -> Option<&mut Vec<String>> {
        self.sections.get_mut("[Proxy Group]")
    }

    fn rule(&self) -> Option<&Vec<String>> {
        self.sections.get("[Rule]")
    }
}

impl Display for SurgeProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.sections
                .values()
                .map(|section| section.join("\n"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

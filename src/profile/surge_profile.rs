use std::fmt::{Display, Formatter};

use crate::config::surge_config::SurgeConfig;
use crate::profile::clash_profile::ProxyGroupType;
use crate::profile::rule::RuleType;
use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::region::Region;
use crate::subscription::url_builder::UrlBuilder;
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
    pub fn new(content: String) -> Self {
        let mut content = content.lines().map(|s| s.to_string()).collect::<Vec<String>>();
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

    pub fn optimize(&mut self, url_builder: UrlBuilder) -> color_eyre::Result<()> {
        self.replace_header(url_builder)?;
        self.optimize_proxies();
        Ok(())
    }

    fn replace_header(&mut self, url_builder: UrlBuilder) -> color_eyre::Result<()> {
        let header = &mut self.sections[MANAGED_CONFIG_HEADER];
        let url = url_builder.build_convertor_url("surge")?;
        header[0] = SurgeConfig::build_managed_config_header(url);
        Ok(())
    }

    fn optimize_proxies(&mut self) {
        let Some(proxies) = self.proxies() else {
            return;
        };
        let proxies = proxies
            .iter()
            .skip(1)
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.split_once('=').map(|(name, _)| name.trim()))
            .collect::<Vec<_>>();
        let (region_map, infos) = group_by_region(&proxies);
        let boslife_group = Self::build_proxy_group(
            "BosLife",
            ProxyGroupType::Select,
            region_map
                .keys()
                .map(|r| format!("{} {}", r.icon, r.cn))
                .collect::<Vec<_>>(),
        );
        let boslife_info = Self::build_proxy_group(
            "BosLife Info",
            ProxyGroupType::Select,
            infos
                .iter()
                .filter_map(|p| p.split_once('=').map(|(name, _)| name.trim()))
                .collect::<Vec<_>>(),
        );
        let region_groups = region_map
            .into_iter()
            .map(|(region, proxies)| {
                let name = format!("{} {}", region.icon, region.cn);
                Self::build_proxy_group(
                    name,
                    ProxyGroupType::UrlTest,
                    proxies
                        .iter()
                        .filter_map(|p| p.split_once('=').map(|(name, _)| name.trim()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        if let Some(section) = self.proxy_groups_mut() {
            section.truncate(1);
            section.push(boslife_group);
            section.push(boslife_info);
            section.extend(region_groups);
            section.push("".to_string());
            section.push("".to_string());
        }
    }

    pub fn generate_rule_provider(&self, policy: RuleSetPolicy, sub_host: impl AsRef<str>) -> Option<String> {
        let Some(rules) = self.rules().filter(|r| !r.is_empty()) else {
            return None;
        };
        let mut matched_rules = rules
            .iter()
            .filter(|rule| {
                let rule_parts = rule.splitn(3, ',').map(str::trim).collect::<Vec<_>>();
                if rule_parts.len() < 2 {
                    return false;
                }
                let rule_type = rule_parts[0];
                let (value, policies) = if rule_parts.len() == 2 {
                    (None, rule_parts[1])
                } else {
                    (Some(rule_parts[1]), rule_parts[2])
                };
                if matches!(policy, RuleSetPolicy::BosLifeSubscription) {
                    value.map(|v| v.contains(sub_host.as_ref())) == Some(true)
                } else if rule_type != RuleType::Final.as_str()
                    && rule_type != RuleType::GeoIP.as_str()
                    && rule_type != RuleType::Match.as_str()
                {
                    policies == policy.as_policies()
                } else {
                    false
                }
            })
            .map(|rule| rule.to_string())
            .collect::<Vec<_>>();
        matched_rules.insert(0, format!("// {}", policy.section_name()));
        Some(matched_rules.join("\n"))
    }

    fn proxies(&self) -> Option<&Vec<String>> {
        self.sections.get("[Proxy]")
    }

    fn proxy_groups_mut(&mut self) -> Option<&mut Vec<String>> {
        self.sections.get_mut("[Proxy Group]")
    }

    fn rules(&self) -> Option<&Vec<String>> {
        self.sections.get("[Rule]")
    }

    fn build_proxy_group<T, I>(name: impl AsRef<str>, r#type: ProxyGroupType, proxies: I) -> String
    where
        T: AsRef<str>,
        I: IntoIterator<Item = T>,
    {
        format!(
            "{} = {}, {}",
            name.as_ref(),
            r#type.serialize(),
            proxies
                .into_iter()
                .map(|p| p.as_ref().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
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

pub fn group_by_region<'a>(proxies: &'a [&str]) -> (IndexMap<&'static Region, Vec<&'a str>>, Vec<&'a str>) {
    let match_number = Regex::new(r"^\d+$").unwrap();
    proxies
        .iter()
        .fold((IndexMap::new(), Vec::new()), |(mut regions, mut infos), proxy| {
            let Some((name, _)) = proxy.split_once('=') else {
                infos.push(proxy);
                return (regions, infos);
            };
            let mut parts = name.split(' ').collect::<Vec<_>>();
            parts.retain(|part| match_number.is_match(part));
            match parts.iter().find_map(Region::detect) {
                Some(region) => regions.entry(region).or_default().push(proxy),
                None => infos.push(proxy),
            }
            (regions, infos)
        })
}

use crate::region::REGIONS;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::{Display, Formatter};

static SECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\[[^\[\]]+]$"#).unwrap());

#[derive(Debug)]
pub struct SurgeProfile {
    sections: IndexMap<String, Vec<String>>,
}

impl SurgeProfile {
    pub fn new(content: impl Into<String>) -> Self {
        let mut content = content.into().lines().map(|s| s.to_string()).collect::<Vec<String>>();
        let mut sections = IndexMap::new();

        let mut section = "header".to_string();
        let mut len = 0;
        while len < content.len() {
            let line = &content[len];
            if SECTION_REGEX.is_match(line) {
                let new_section = line.to_string();
                sections.insert(
                    std::mem::replace(&mut section, new_section),
                    content.drain(..len).collect::<Vec<_>>(),
                );
                len = 0;
            }
            len += 1;
        }
        sections.insert(section, content);

        Self { sections }
    }

    pub fn replace_header(&mut self) {
        let header = &mut self.sections["header"];
        header[0] = "#!MANAGED-CONFIG http://mini-lan.sgponte:8196/surge?url=https%3A%2F%2Fapi.imlax.net%2Fapi%2Fv1%2Fclient%2Fsubscribe%3Ftoken%3D4287df4ba2618c4da1f9efd7ffeb30a5&flag=surge&secret=bppleman interval=259200 strict=true".to_string();
    }

    pub fn organize_proxy_group(&mut self) {
        let mut proxy_map: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut boslife_info = vec![];
        if let Some(proxies) = self.proxy() {
            proxies[1..].iter().filter(|p| !p.is_empty()).for_each(|proxy| {
                if proxy.starts_with("[BosLife]") {
                    if let Some(region) = REGIONS.iter().find(|region| proxy.contains(region.as_str())) {
                        proxy_map
                            .entry(region.to_string())
                            .or_default()
                            .push(proxy.split('=').next().unwrap().to_string())
                    }
                } else {
                    boslife_info.push(proxy.split('=').next().unwrap().to_string())
                }
            })
        }
        let boslife_group = format!(
            "BosLife = select, {}",
            proxy_map.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(", ")
        );
        let boslife_info = format!("BosLife Info = select, {}", boslife_info.join(", "));
        if let Some(proxy_group) = self.proxy_group_mut() {
            if !proxy_group.is_empty() {
                proxy_group.truncate(1);
            }
            proxy_group.push(boslife_group);
            proxy_group.push(boslife_info);
            proxy_map
                .into_iter()
                .filter(|(name, _)| name != "BosLife Info")
                .for_each(|(name, proxies)| {
                    proxy_group.push(format!("{name} = url-test, {}", proxies.join(", ")));
                });
            proxy_group.push("".to_string());
            proxy_group.push("".to_string());
        }
    }

    pub fn extract_rule(&self, policies: &[impl AsRef<str>]) -> String {
        self.rule()
            .map(|rules| {
                rules
                    .iter()
                    .filter(|rule| !rule.starts_with("//"))
                    .filter(|rule| !rule.starts_with('#'))
                    .filter(|rule| !rule.starts_with("GEOIP,CN"))
                    .filter(|rule| !rule.starts_with("FINAL,DIRECT"))
                    .filter(|rule| policies.iter().all(|policy| rule.contains(policy.as_ref())))
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

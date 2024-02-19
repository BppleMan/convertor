use std::fmt::{Display, Formatter};

use crate::profile::group_by;
use crate::region::Region;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;

static SECTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\[[^\[\]]+]$"#).unwrap());

#[derive(Debug)]
pub struct SurgeProfile {
    sections: IndexMap<String, Vec<String>>,
}

impl SurgeProfile {
    pub fn new(content: impl Into<String>) -> Self {
        let mut content = content
            .into()
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
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

    pub fn replace_header(&mut self, host: Option<impl AsRef<str>>) {
        let host = host
            .as_ref()
            .map(|h| h.as_ref())
            .unwrap_or("mini-lan.sgponte");
        let header = &mut self.sections["header"];
        header[0] = format!("#!MANAGED-CONFIG http://{}/surge?url=https%3A%2F%2Fapi.imlax.net%2Fapi%2Fv1%2Fclient%2Fsubscribe%3Ftoken%3D4287df4ba2618c4da1f9efd7ffeb30a5&secret=bppleman interval=259200 strict=true", host)
    }

    pub fn organize_proxy_group(&mut self) {
        let mut proxy_map: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut boslife = vec![];
        let mut boslife_info = vec![];
        if let Some(proxies) = self.proxy() {
            let proxies = proxies
                .iter()
                .filter(|p| !p.is_empty())
                .map(|p| p.split('=').next().unwrap().to_string())
                .collect::<Vec<_>>();
            boslife_info = proxies[1..4].to_vec();
            proxy_map = group_by(&proxies[4..]);
            let groups = group_by(
                &proxy_map.keys().map(|k| k.as_str()).collect::<Vec<_>>(),
            );
            for (group, proxies) in &groups {
                boslife.push(group.clone());
                if !proxy_map.contains_key(group) {
                    proxy_map.insert(group.clone(), proxies.clone());
                }
            }
            for (group, proxies) in groups {
                if let Some(region) = Region::detect(group) {
                    boslife.push(region.cn.clone());
                    if !proxy_map.contains_key(&region.cn) {
                        proxy_map.insert(region.cn.clone(), proxies);
                    }
                }
            }
        }
        let boslife_group = format!("BosLife = select, {}", boslife.join(", "));
        let boslife_info =
            format!("BosLife Info = select, {}", boslife_info.join(", "));
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
                    proxy_group.push(format!(
                        "{name} = url-test, {}",
                        proxies.join(", ")
                    ));
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
                    .filter(|rule| {
                        policies
                            .iter()
                            .all(|policy| rule.contains(policy.as_ref()))
                    })
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

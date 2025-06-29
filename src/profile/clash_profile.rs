use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use tracing::warn;

mod proxy;
mod proxy_group;
mod rule;
mod rule_provider;

use crate::profile::clash_profile::rule::{Rule, RuleType};
use crate::profile::clash_profile::rule_provider::RuleProvider;
use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::region::Region;
use crate::subscription::url_builder::UrlBuilder;
pub use proxy::*;
pub use proxy_group::*;

const TEMPLATE_STR: &str = include_str!("../../assets/clash/template.yaml");

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
    pub rules: Option<Vec<Rule>>,
    #[serde(rename = "rule-providers")]
    pub rule_providers: Option<Vec<(RuleSetPolicy, RuleProvider)>>,
}

impl ClashProfile {
    pub fn template() -> color_eyre::Result<Self> {
        Ok(serde_yaml::from_str(TEMPLATE_STR)?)
    }

    pub fn generate_profile(
        &mut self,
        raw_profile: String,
        sub_host: impl AsRef<str>,
        url_builder: &UrlBuilder,
    ) -> color_eyre::Result<()> {
        let raw_profile: ClashProfile = serde_yaml::from_str(&raw_profile)?;
        self.proxies = raw_profile.proxies;
        self.rules = raw_profile.rules;
        self.optimize_proxies();
        self.optimize_rules(sub_host, url_builder);
        Ok(())
    }

    fn optimize_proxies(&mut self) {
        let (region_map, infos) = group_by_region(&self.proxies);
        let boslife_group = ProxyGroup::new(
            "BosLife".to_string(),
            ProxyGroupType::Select,
            region_map.keys().map(|r| format!("{} {}", r.icon, r.cn)).collect(),
        );
        let boslife_info = ProxyGroup::new(
            "BosLife Info".to_string(),
            ProxyGroupType::Select,
            infos
                .into_iter()
                .map(|proxy| proxy.name.to_string())
                .collect::<Vec<_>>(),
        );
        let region_groups = region_map
            .into_iter()
            .map(|(region, indices)| {
                let proxies = indices.into_iter().map(|proxy| proxy.name.to_string()).collect();
                ProxyGroup::new(
                    format!("{} {}", region.icon, region.cn),
                    ProxyGroupType::UrlTest,
                    proxies,
                )
            })
            .collect::<Vec<_>>();

        self.proxy_groups.clear();
        self.proxy_groups.push(boslife_group);
        self.proxy_groups.push(boslife_info);
        for group in region_groups {
            self.proxy_groups.push(group);
        }
    }

    fn optimize_rules(&mut self, sub_host: impl AsRef<str>, url_builder: &UrlBuilder) {
        let Some(rules) = &mut self.rules else {
            return;
        };
        let mut retain = vec![];
        let rsp_set = rules
            .drain(..)
            .filter_map(|rule| {
                if matches!(rule.rule_type, RuleType::GeoIP | RuleType::Final | RuleType::Match) || rule.value.is_none()
                {
                    retain.push(rule);
                    return None;
                }
                let value = rule.value.as_ref()?;
                let rsp = if value.contains(sub_host.as_ref()) {
                    Some(RuleSetPolicy::BosLifeSubscription)
                } else {
                    match RuleSetPolicy::from_str(&rule.policies) {
                        Ok(rsp) => Some(rsp),
                        Err(e) => {
                            warn!("{e}");
                            None
                        }
                    }
                };
                if rsp.is_none() {
                    retain.push(rule);
                }
                rsp
            })
            .collect::<HashSet<_>>();

        let mut rsp_list = rsp_set.into_iter().collect::<Vec<_>>();
        rsp_list.sort();

        let mut rules = rsp_list.iter().copied().map(Rule::new_rule_set).collect::<Vec<_>>();
        rules.extend(retain);
        self.rules.replace(rules);

        let rule_providers = rsp_list
            .iter()
            .flat_map(|policy| {
                let Ok(url) = url_builder.build_rule_set_url("clash", policy) else {
                    warn!("无法构建规则集 URL, 可能是订阅 URL 错误");
                    return None;
                };
                let provider_name = policy.provider_name();
                Some((*policy, RuleProvider::new(url, provider_name)))
            })
            .collect::<Vec<(RuleSetPolicy, RuleProvider)>>();
        self.rule_providers.replace(rule_providers);
    }

    pub fn generate_rule_provider(&self, policy: RuleSetPolicy, sub_host: impl AsRef<str>) -> Option<String> {
        if let Some(rules) = self.rules.as_ref().filter(|r| !r.is_empty()) {
            let mut matched_rules = rules
                .iter()
                .filter(|rule| {
                    if matches!(policy, RuleSetPolicy::BosLifeSubscription) {
                        // BosLifeSubscription 策略只包括机场订阅链接
                        rule.value.as_ref().map(|v| v.contains(sub_host.as_ref())) == Some(true)
                    } else {
                        // 对于其他策略，检查规则的 policies 是否匹配，但不能是 Final 或 Match 类型
                        rule.policies == policy.policy() && !matches!(rule.rule_type, RuleType::Final | RuleType::Match)
                    }
                })
                .map(|rule| format!(r#"    {}"#, rule.serialize()))
                .collect::<Vec<_>>();
            matched_rules.insert(0, "payload:".to_string());
            Some(matched_rules.join("\n"))
        } else {
            None
        }
    }

    pub fn serialize(&self) -> String {
        let mut field = vec![
            format!(r#"port: {}"#, self.port),
            format!(r#""socks-port": {}"#, self.socks_port),
            format!(r#""redir-port": {}"#, self.redir_port),
            format!(r#""allow-lan": {}"#, self.allow_lan),
            format!(r#""mode": "{}""#, self.mode),
            format!(r#""log-level": "{}""#, self.log_level),
            format!(r#""external-controller": "{}""#, self.external_controller),
            format!(r#""secret": "{}""#, self.secret),
        ];
        field.push("proxies:".to_string());
        field.extend(self.proxies.iter().map(|p| format!("    {}", p.serialize())));
        field.push("proxy-groups:".to_string());
        field.extend(self.proxy_groups.iter().map(|g| format!("    {}", g.serialize())));
        if let Some(rule_providers) = self.rule_providers.as_ref().filter(|r| !r.is_empty()) {
            field.push("rule-providers:".to_string());
            field.extend(
                rule_providers
                    .iter()
                    .map(|(policy, provider)| format!("    {}: {}", policy.provider_name(), provider.serialize())),
            );
        }
        if let Some(rules) = self.rules.as_ref().filter(|r| !r.is_empty()) {
            field.push("rules:".to_string());
            field.extend(rules.iter().map(|r| format!("    {}", r.serialize())));
        }
        format!("{}\n", field.join("\n"))
    }
}

impl FromStr for ClashProfile {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_yaml::from_str(s)?)
    }
}

fn group_by_region(proxies: &[Proxy]) -> (IndexMap<&'static Region, Vec<&Proxy>>, Vec<&Proxy>) {
    let match_number = Regex::new(r"^\d+$").unwrap();
    proxies
        .iter()
        .fold((IndexMap::new(), Vec::new()), |(mut regions, mut infos), proxy| {
            let mut parts = proxy.name.split(' ').collect::<Vec<_>>();
            parts.retain(|part| !match_number.is_match(part));
            match parts.iter().find_map(Region::detect) {
                Some(region) => regions.entry(region).or_default().push(proxy),
                None => infos.push(proxy),
            }
            (regions, infos)
        })
}

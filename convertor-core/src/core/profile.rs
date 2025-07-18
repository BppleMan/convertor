use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::rule::Rule;
use crate::core::region::Region;
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashSet;

pub mod policy_preset;
pub mod proxy;
pub mod proxy_group;
pub mod rule;
pub mod rule_provider;
pub mod policy;
pub mod profile;
pub mod surge_profile;
pub mod clash_profile;

pub(super) fn group_by_region(proxies: &[Proxy]) -> (IndexMap<&'static Region, Vec<&Proxy>>, Vec<&Proxy>) {
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

pub fn extract_policies(rules: &[Rule]) -> Vec<Policy> {
    let mut policies = rules
        .iter()
        .filter_map(|rule| {
            if rule.policy.is_built_in() {
                None
            } else {
                let mut policy = rule.policy.clone();
                policy.option = None;
                policy.is_subscription = false;
                Some(policy)
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    policies.sort();
    policies
}

pub fn extract_policies_for_rule_provider(rules: &[Rule], sub_host: impl AsRef<str>) -> Vec<Policy> {
    let mut policies = rules
        .iter()
        .filter_map(|rule| {
            if rule
                .value
                .as_ref()
                .map(|v| v.contains(sub_host.as_ref()))
                .unwrap_or(false)
            {
                Some(Policy::subscription_policy())
            } else if !rule.policy.is_built_in() {
                Some(rule.policy.clone())
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    policies.sort();
    policies
}

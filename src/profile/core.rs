use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::rule::Rule;
use crate::region::Region;
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashSet;

pub mod policy_preset;
pub mod proxy;
pub mod proxy_group;
pub mod rule;
pub mod rule_provider;
pub mod policy;

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
    rules
        .iter()
        .map(|rule| {
            let mut policy = rule.policy.clone();
            policy.option = None;
            policy.is_subscription = false;
            policy
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .filter(|policy| !policy.is_built_in())
        .collect()
}

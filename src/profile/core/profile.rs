use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::profile::core::rule::{ProviderRule, Rule};
use crate::profile::core::{extract_policies, group_by_region};
use crate::url_builder::UrlBuilder;
use color_eyre::Result;
use std::collections::HashMap;
use tracing::{instrument, warn};

pub trait Profile {
    type PROFILE;

    fn proxies(&self) -> &[Proxy];

    fn proxies_mut(&mut self) -> &mut Vec<Proxy>;

    fn proxy_groups(&self) -> &[ProxyGroup];

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup>;

    fn rules(&self) -> &[Rule];

    fn rules_mut(&mut self) -> &mut Vec<Rule>;

    fn policy_of_rules(&self) -> &HashMap<Policy, Vec<ProviderRule>>;

    fn policy_of_rules_mut(&mut self) -> &mut HashMap<Policy, Vec<ProviderRule>>;

    fn parse(content: String) -> Result<Self::PROFILE>;

    fn optimize(
        &mut self,
        url_builder: &UrlBuilder,
        raw_profile: Option<String>,
        secret: Option<impl AsRef<str>>,
    ) -> Result<()>;

    #[instrument(skip_all)]
    fn optimize_proxies(&mut self) -> Result<()> {
        if self.proxies().is_empty() {
            return Ok(());
        };
        let (region_map, infos) = group_by_region(&self.proxies());
        // 一个包含了所有地区组的大型代理组
        let region_list = region_map.keys().map(|r| r.policy_name()).collect::<Vec<_>>();
        let policies = extract_policies(&self.rules());
        let policy_groups = policies
            .iter()
            .map(|policy| {
                let name = policy.name.clone();
                ProxyGroup::new(name, ProxyGroupType::Select, region_list.clone())
            })
            .collect::<Vec<_>>();
        let convertor_group = ProxyGroup::new(
            "Convertor".to_string(),
            ProxyGroupType::Select,
            infos.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>(),
        );
        // 每个地区的地区代理组
        let region_groups = region_map
            .into_iter()
            .map(|(region, proxies)| {
                let name = format!("{} {}", region.icon, region.cn);
                ProxyGroup::new(
                    name,
                    ProxyGroupType::Smart,
                    proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        self.proxy_groups_mut().clear();
        self.proxy_groups_mut().extend(policy_groups);
        self.proxy_groups_mut().push(convertor_group);
        self.proxy_groups_mut().extend(region_groups);

        Ok(())
    }

    #[instrument(skip_all)]
    fn optimize_rules(&mut self, url_builder: &UrlBuilder) -> Result<()> {
        let sub_host = url_builder.sub_host()?;
        let (built_in_rules, other_rules): (Vec<Rule>, Vec<Rule>) = self
            .rules_mut()
            .drain(..)
            .partition(|rule| rule.is_built_in() || rule.value.is_none());

        for mut rule in other_rules {
            if rule.value.as_ref().map(|v| v.contains(&sub_host)) == Some(true) {
                rule.policy.is_subscription = true;
                self.policy_of_rules_mut()
                    .entry(Policy::subscription_policy())
                    .or_default()
                    .push(rule.try_into()?);
            } else if rule.value.is_some() {
                self.policy_of_rules_mut()
                    .entry(rule.policy.clone())
                    .or_default()
                    .push(rule.try_into()?);
            } else {
                warn!("规则 {:?} 没有值，无法添加到策略中", rule);
            }
        }

        let mut policy_list = self.policy_of_rules().keys().cloned().collect::<Vec<_>>();
        policy_list.sort();

        for policy in policy_list {
            if let Err(e) = self.append_rule_provider(&policy, url_builder) {
                warn!("无法为 {:?} 添加 Rule Provider: {}", policy, e);
            }
        }

        self.rules_mut().extend(built_in_rules);

        Ok(())
    }

    fn append_rule_provider(&mut self, policy: &Policy, url_builder: &UrlBuilder) -> Result<()>;

    #[instrument(skip_all)]
    fn get_provider_rules_with_policy(&self, policy: &Policy) -> Option<&Vec<ProviderRule>> {
        self.policy_of_rules().get(policy)
    }
}

use crate::client::Client;
use crate::core::error::ParseError;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::profile::{extract_policies, group_by_region};
use crate::core::result::ParseResult;
use crate::url_builder::UrlBuilder;
use std::collections::HashMap;
use tracing::{instrument, span, warn};

pub trait Profile {
    type PROFILE;

    fn client() -> Client;

    fn proxies(&self) -> &[Proxy];

    fn proxies_mut(&mut self) -> &mut Vec<Proxy>;

    fn proxy_groups(&self) -> &[ProxyGroup];

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup>;

    fn rules(&self) -> &[Rule];

    fn rules_mut(&mut self) -> &mut Vec<Rule>;

    fn policy_of_rules(&self) -> &HashMap<Policy, Vec<ProviderRule>>;

    fn policy_of_rules_mut(&mut self) -> &mut HashMap<Policy, Vec<ProviderRule>>;

    fn parse(content: String) -> ParseResult<Self::PROFILE>;

    fn merge(&mut self, profile: Self::PROFILE, secret: impl AsRef<str>) -> ParseResult<()>;

    fn optimize(&mut self, url_builder: &UrlBuilder) -> ParseResult<()>;

    #[instrument(skip_all)]
    fn optimize_proxies(&mut self) -> ParseResult<()> {
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
                let proxy_group_type = match Self::client() {
                    Client::Surge => ProxyGroupType::Smart,
                    Client::Clash => ProxyGroupType::UrlTest,
                };
                let proxies = proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>();
                ProxyGroup::new(name, proxy_group_type, proxies)
            })
            .collect::<Vec<_>>();
        self.proxy_groups_mut().clear();
        self.proxy_groups_mut().extend(policy_groups);
        self.proxy_groups_mut().push(convertor_group);
        self.proxy_groups_mut().extend(region_groups);

        Ok(())
    }

    #[instrument(skip_all)]
    fn optimize_rules(&mut self, url_builder: &UrlBuilder) -> ParseResult<()> {
        let sub_host = url_builder.sub_host().map_err(|_| ParseError::SubHost)?;
        let inner_span = span!(tracing::Level::INFO, "拆分内置规则和其他规则");
        let _guard = inner_span.entered();
        let (built_in_rules, other_rules): (Vec<Rule>, Vec<Rule>) = self
            .rules_mut()
            .drain(..)
            .partition(|rule| rule.is_built_in() || rule.value.is_none());
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "处理其它规则");
        let _guard = inner_span.entered();
        for mut rule in other_rules {
            if rule.value.as_ref().map(|v| v.contains(&sub_host)) == Some(true) {
                let inner_span = span!(tracing::Level::INFO, "Rule 转换为 ProviderRule");
                let _inner_guard = inner_span.entered();
                rule.policy.is_subscription = true;
                drop(_inner_guard);

                let inner_span = span!(tracing::Level::INFO, "将规则添加到订阅策略");
                let _inner_guard = inner_span.entered();
                self.policy_of_rules_mut()
                    .entry(Policy::subscription_policy())
                    .or_default()
                    .push(rule.try_into()?);
                drop(_inner_guard);
            } else if rule.value.is_some() {
                let inner_span = span!(tracing::Level::INFO, "将规则添加到策略");
                let _inner_guard = inner_span.entered();
                self.policy_of_rules_mut()
                    .entry(rule.policy.clone())
                    .or_default()
                    .push(rule.try_into()?);
                drop(_inner_guard);
            } else {
                warn!("规则 {:?} 没有值，无法添加到策略中", rule);
            }
        }
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "排序策略列表");
        let _guard = inner_span.entered();
        let mut policy_list = self.policy_of_rules().keys().cloned().collect::<Vec<_>>();
        policy_list.sort();
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "为每个策略添加规则提供者");
        let _guard = inner_span.entered();
        for policy in policy_list {
            if let Err(e) = self.append_rule_provider(&policy, url_builder) {
                warn!("无法为 {:?} 添加 Rule Provider: {}", policy, e);
            }
        }
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "拼接内置规则");
        let _guard = inner_span.entered();
        self.rules_mut().extend(built_in_rules);
        drop(_guard);

        Ok(())
    }

    fn append_rule_provider(&mut self, policy: &Policy, url_builder: &UrlBuilder) -> ParseResult<()>;

    #[instrument(skip_all)]
    fn get_provider_rules_with_policy(&self, policy: &Policy) -> Option<&Vec<ProviderRule>> {
        self.policy_of_rules().get(policy)
    }
}

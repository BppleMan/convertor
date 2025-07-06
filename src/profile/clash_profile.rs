use crate::client::Client;
use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::profile::core::rule::{Rule, RuleType};
use crate::profile::core::rule_provider::RuleProvider;
use crate::profile::core::{extract_policies, group_by_region};
use crate::profile::parser::clash_parser::ClashParser;
use crate::profile::renderer::clash_renderer::ClashRenderer;
use crate::subscription::url_builder::UrlBuilder;
use serde::Deserialize;
use std::collections::HashSet;
use std::str::FromStr;
use tracing::{instrument, warn};

const TEMPLATE_STR: &str = include_str!("../../assets/clash/template.yaml");

#[derive(Debug, Deserialize)]
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
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(rename = "rule-providers", default)]
    pub rule_providers: Vec<(String, RuleProvider)>,
}

impl ClashProfile {
    #[instrument(skip_all)]
    pub fn parse(content: String) -> color_eyre::Result<Self> {
        Ok(ClashParser::parse(content)?)
    }

    #[instrument(skip_all)]
    pub fn template() -> color_eyre::Result<Self> {
        Ok(ClashParser::parse(TEMPLATE_STR)?)
    }

    #[instrument(skip_all)]
    pub fn optimize(
        &mut self,
        url_builder: &UrlBuilder,
        raw_profile: String,
        secret: impl AsRef<str>,
    ) -> color_eyre::Result<()> {
        let raw_profile = ClashProfile::parse(raw_profile)?;
        let sub_host = url_builder.sub_host()?;
        self.proxies = raw_profile.proxies;
        self.rules = raw_profile.rules;
        self.secret = secret.as_ref().to_string();
        self.optimize_proxies();
        self.optimize_rules(sub_host, url_builder)?;
        Ok(())
    }

    fn optimize_proxies(&mut self) {
        let (region_map, infos) = group_by_region(&self.proxies);
        // 一个包含了所有地区组的大型代理组
        let region_list = region_map.keys().map(|r| r.policy_name()).collect::<Vec<_>>();
        let policies = extract_policies(&self.rules);
        let policy_groups = policies
            .iter()
            .map(|policy| {
                let name = policy.name.clone();
                ProxyGroup::new(name, ProxyGroupType::Select, region_list.clone())
            })
            .collect::<Vec<_>>();
        let subscription_info = ProxyGroup::new(
            "Subscription Info".to_string(),
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
                    ProxyGroupType::UrlTest,
                    proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        self.proxy_groups.clear();
        self.proxy_groups.extend(policy_groups);
        self.proxy_groups.push(subscription_info);
        self.proxy_groups.extend(region_groups);
    }

    fn optimize_rules(&mut self, sub_host: impl AsRef<str>, url_builder: &UrlBuilder) -> color_eyre::Result<()> {
        let mut retain = vec![];
        let policy_set = self
            .rules
            .drain(..)
            .filter_map(|rule| {
                if matches!(rule.rule_type, RuleType::GeoIP | RuleType::Final | RuleType::Match) || rule.value.is_none()
                {
                    retain.push(rule);
                    return None;
                }
                let value = rule.value.as_ref()?;
                let policy = if value.contains(sub_host.as_ref()) {
                    Policy::subscription_policy()
                } else {
                    rule.policy
                };
                Some(policy)
            })
            .collect::<HashSet<_>>();

        let mut policy_list = policy_set.into_iter().collect::<Vec<_>>();
        policy_list.sort();

        let mut rules = policy_list
            .iter()
            .map(Rule::clash_rule_provider)
            .collect::<color_eyre::Result<Vec<_>>>()?;
        rules.extend(retain);
        self.rules.extend(rules);

        self.rule_providers = policy_list
            .into_iter()
            .filter_map(|policy| {
                let Ok(url) = url_builder.build_rule_provider_url(Client::Clash, &policy) else {
                    warn!("无法构建 Rule Provider URL, 可能是订阅 URL 错误");
                    return None;
                };
                let provider_name = ClashRenderer::render_provider_name_from_policy(&policy).ok()?;
                Some((provider_name.clone(), RuleProvider::new(url, provider_name)))
            })
            .collect::<Vec<(_, _)>>();

        Ok(())
    }

    pub fn rules_for_provider(&self, policy: Policy, sub_host: impl AsRef<str>) -> color_eyre::Result<Vec<Rule>> {
        let rules = self
            .rules
            .iter()
            .filter(|rule| {
                if policy == Policy::subscription_policy() {
                    // Subscription 策略只包括机场订阅链接
                    rule.value.as_ref().map(|v| v.contains(sub_host.as_ref())) == Some(true)
                } else if !matches!(rule.rule_type, RuleType::Final | RuleType::GeoIP | RuleType::Match) {
                    // 对于其他策略，检查规则的 policies 是否匹配，但不能是 Final 或 Match 类型
                    rule.policy == policy && rule.value.as_ref().map(|v| v.contains(sub_host.as_ref())) != Some(true)
                } else {
                    false
                }
            })
            .map(|rule| {
                let mut rule = rule.clone();
                rule.policy.is_subscription = true;
                rule
            })
            .collect::<Vec<_>>();
        Ok(rules)
    }
}

impl FromStr for ClashProfile {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ClashParser::parse(s)?)
    }
}

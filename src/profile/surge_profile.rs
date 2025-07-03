use crate::client::Client;
use crate::config::surge_config::SurgeConfig;
use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::profile::core::rule::{Rule, RuleType};
use crate::profile::core::{extract_policies, group_by_region};
use crate::profile::parser::surge_parser::SurgeParser;
use crate::subscription::url_builder::UrlBuilder;
use indexmap::IndexMap;
use tracing::instrument;

#[derive(Debug)]
pub struct SurgeProfile {
    pub header: String,
    pub general: Vec<String>,
    pub proxies: Vec<Proxy>,
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<Rule>,
    pub url_rewrite: Vec<String>,
    pub misc: IndexMap<String, Vec<String>>,
}

impl SurgeProfile {
    #[instrument(skip_all)]
    pub fn parse(content: String) -> color_eyre::Result<Self> {
        Ok(SurgeParser::parse_profile(content)?)
    }

    pub fn optimize(&mut self, url_builder: UrlBuilder) -> color_eyre::Result<()> {
        self.replace_header(url_builder)?;
        self.optimize_proxies();
        Ok(())
    }

    fn replace_header(&mut self, url_builder: UrlBuilder) -> color_eyre::Result<()> {
        let url = url_builder.build_convertor_url(Client::Surge)?;
        self.header = SurgeConfig::build_managed_config_header(url);
        Ok(())
    }

    fn optimize_proxies(&mut self) {
        if self.proxies.is_empty() {
            return;
        };
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
                    ProxyGroupType::Smart,
                    proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        self.proxy_groups.clear();
        self.proxy_groups.extend(policy_groups);
        self.proxy_groups.push(subscription_info);
        self.proxy_groups.extend(region_groups);
    }

    pub fn rules_for_provider(&self, policy: Policy, sub_host: impl AsRef<str>) -> Vec<Rule> {
        self.rules
            .iter()
            .filter(|rule| {
                if policy == Policy::subscription_policy() {
                    println!("{:?}, {}", rule.value, sub_host.as_ref());
                    rule.value.as_ref().map(|v| v.contains(sub_host.as_ref())) == Some(true)
                } else if !matches!(rule.rule_type, RuleType::Final | RuleType::GeoIP | RuleType::Match) {
                    policy == rule.policy
                } else {
                    false
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    }
}

use crate::config::url_builder::UrlBuilder;
use crate::region::Region;
use indexmap::IndexMap;
use regex::Regex;

pub mod surge_profile;
pub mod clash_profile;

pub fn group_by_region<S: AsRef<str>>(
    sources: &[S],
) -> IndexMap<String, Vec<String>> {
    let match_number = Regex::new(r"\W*\d+\s*$").unwrap();
    sources.iter().fold(
        IndexMap::<String, Vec<String>>::new(),
        |mut acc, source| {
            let source = source.as_ref();
            let region_part = match_number.replace(source, "").to_string();
            acc.entry(region_part).or_default().push(source.to_string());
            acc
        },
    )
}

pub fn split_and_merge_groups(
    groups: IndexMap<String, Vec<String>>,
) -> (IndexMap<&'static Region, Vec<String>>, Vec<String>) {
    let mut useful_groups: IndexMap<&'static Region, Vec<String>> =
        IndexMap::new();
    let mut extra_groups = vec![];

    for group_name in groups.keys() {
        if let Some(region) = Region::detect(group_name) {
            useful_groups
                .entry(region)
                .or_default()
                .extend(groups[group_name].clone());
        } else {
            extra_groups.extend(groups[group_name].clone());
        }
    }

    (useful_groups, extra_groups)
}

pub trait Profile {
    fn generate_profile(url_builder: &UrlBuilder)
        -> color_eyre::Result<String>;
    /// 替换配置头, 主要是处理 MANAGED-CONFIG, clash config 应该不需要做额外操作
    fn replace_header(&mut self, convertor_url: impl AsRef<str>);
    fn organize_proxy_group(&mut self);
    fn extract_rule(&self, policies: impl AsRef<str>) -> String;
}

pub enum RuleSetPolicy {
    BosLifeSubscription,
    BosLifePolicy,
    BosLifeNoResolvePolicy,
    BosLifeForceRemoteDnsPolicy,
    DirectPolicy,
    DirectNoResolvePolicy,
    DirectForceRemoteDnsPolicy,
}

impl RuleSetPolicy {
    pub const fn all() -> &'static [RuleSetPolicy] {
        &[
            RuleSetPolicy::BosLifeSubscription,
            RuleSetPolicy::BosLifePolicy,
            RuleSetPolicy::BosLifeNoResolvePolicy,
            RuleSetPolicy::BosLifeForceRemoteDnsPolicy,
            RuleSetPolicy::DirectPolicy,
            RuleSetPolicy::DirectNoResolvePolicy,
            RuleSetPolicy::DirectForceRemoteDnsPolicy,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RuleSetPolicy::BosLifeSubscription => "[BosLife Subscription]",
            RuleSetPolicy::BosLifePolicy => "[BosLife Policy]",
            RuleSetPolicy::BosLifeNoResolvePolicy => {
                "[BosLife No Resolve Policy]"
            }
            RuleSetPolicy::BosLifeForceRemoteDnsPolicy => {
                "[BosLife Force Remote Dns Policy]"
            }
            RuleSetPolicy::DirectPolicy => "[Direct Policy]",
            RuleSetPolicy::DirectNoResolvePolicy => {
                "[Direct No Resolve Policy]"
            }
            RuleSetPolicy::DirectForceRemoteDnsPolicy => {
                "[Direct Force Remote Dns Policy]"
            }
        }
    }

    pub fn comment(&self) -> String {
        format!(
            r#"// Added for {} by convertor/{}"#,
            self.name(),
            env!("CARGO_PKG_VERSION")
        )
    }

    pub fn policy(&self) -> &'static str {
        match self {
            RuleSetPolicy::BosLifeSubscription => "DIRECT",
            RuleSetPolicy::BosLifePolicy => "BosLife",
            RuleSetPolicy::BosLifeNoResolvePolicy => "BosLife,no-resolve",
            RuleSetPolicy::BosLifeForceRemoteDnsPolicy => {
                "BosLife,force-remote-dns"
            }
            RuleSetPolicy::DirectPolicy => "DIRECT",
            RuleSetPolicy::DirectNoResolvePolicy => "DIRECT,no-resolve",
            RuleSetPolicy::DirectForceRemoteDnsPolicy => {
                "DIRECT,force-remote-dns"
            }
        }
    }

    pub fn rule_set(&self, rule_set_url: impl AsRef<str>) -> String {
        match self {
            RuleSetPolicy::BosLifeSubscription
            | RuleSetPolicy::BosLifePolicy
            | RuleSetPolicy::BosLifeNoResolvePolicy
            | RuleSetPolicy::BosLifeForceRemoteDnsPolicy
            | RuleSetPolicy::DirectPolicy
            | RuleSetPolicy::DirectNoResolvePolicy => format!(
                r#"RULE-SET,{},{} {}"#,
                rule_set_url.as_ref(),
                self.policy(),
                self.comment()
            ),
            RuleSetPolicy::DirectForceRemoteDnsPolicy => format!(
                r#"// RULE-SET,{},{} {}"#,
                rule_set_url.as_ref(),
                self.policy(),
                self.comment()
            ),
        }
    }

    pub fn provider_name(&self) -> &'static str {
        match self {
            RuleSetPolicy::BosLifeSubscription => "BosLifeSubscription",
            RuleSetPolicy::BosLifePolicy => "BosLife",
            RuleSetPolicy::BosLifeNoResolvePolicy => "BosLifeNoResolve",
            RuleSetPolicy::BosLifeForceRemoteDnsPolicy => {
                "BosLifeForceRemoteDns"
            }
            RuleSetPolicy::DirectPolicy => "Direct",
            RuleSetPolicy::DirectNoResolvePolicy => "DirectNoResolve",
            RuleSetPolicy::DirectForceRemoteDnsPolicy => "DirectForceRemoteDns",
        }
    }
}

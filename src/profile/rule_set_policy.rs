use color_eyre::eyre::eyre;
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Serialize, Deserialize)]
pub enum RuleSetPolicy {
    BosLifeSubscription,
    Direct,
    DirectNoResolve,
    DirectForceRemoteDns,
    BosLife,
    BosLifeNoResolve,
    BosLifeForceRemoteDns,
}

impl RuleSetPolicy {
    pub const fn all() -> &'static [RuleSetPolicy] {
        &[
            RuleSetPolicy::BosLifeSubscription,
            RuleSetPolicy::BosLife,
            RuleSetPolicy::BosLifeNoResolve,
            RuleSetPolicy::BosLifeForceRemoteDns,
            RuleSetPolicy::Direct,
            RuleSetPolicy::DirectNoResolve,
            RuleSetPolicy::DirectForceRemoteDns,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RuleSetPolicy::BosLifeSubscription => "[BosLife Subscription]",
            RuleSetPolicy::BosLife => "[BosLife Policy]",
            RuleSetPolicy::BosLifeNoResolve => "[BosLife No Resolve Policy]",
            RuleSetPolicy::BosLifeForceRemoteDns => "[BosLife Force Remote Dns Policy]",
            RuleSetPolicy::Direct => "[Direct Policy]",
            RuleSetPolicy::DirectNoResolve => "[Direct No Resolve Policy]",
            RuleSetPolicy::DirectForceRemoteDns => "[Direct Force Remote Dns Policy]",
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
            RuleSetPolicy::BosLife => "BosLife",
            RuleSetPolicy::BosLifeNoResolve => "BosLife,no-resolve",
            RuleSetPolicy::BosLifeForceRemoteDns => "BosLife,force-remote-dns",
            RuleSetPolicy::Direct => "DIRECT",
            RuleSetPolicy::DirectNoResolve => "DIRECT,no-resolve",
            RuleSetPolicy::DirectForceRemoteDns => "DIRECT,force-remote-dns",
        }
    }

    pub fn rule_set(&self, rule_set_url: impl AsRef<str>) -> String {
        match self {
            RuleSetPolicy::BosLifeSubscription
            | RuleSetPolicy::BosLife
            | RuleSetPolicy::BosLifeNoResolve
            | RuleSetPolicy::BosLifeForceRemoteDns
            | RuleSetPolicy::Direct
            | RuleSetPolicy::DirectNoResolve => format!(
                r#"RULE-SET,{},{} {}"#,
                rule_set_url.as_ref(),
                self.policy(),
                self.comment()
            ),
            RuleSetPolicy::DirectForceRemoteDns => format!(
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
            RuleSetPolicy::BosLife => "BosLife",
            RuleSetPolicy::BosLifeNoResolve => "BosLifeNoResolve",
            RuleSetPolicy::BosLifeForceRemoteDns => "BosLifeForceRemoteDns",
            RuleSetPolicy::Direct => "Direct",
            RuleSetPolicy::DirectNoResolve => "DirectNoResolve",
            RuleSetPolicy::DirectForceRemoteDns => "DirectForceRemoteDns",
        }
    }
}

impl FromStr for RuleSetPolicy {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BosLife" => Ok(RuleSetPolicy::BosLife),
            "BosLife,no-resolve" => Ok(RuleSetPolicy::BosLifeNoResolve),
            "BosLife,force-remote" => Ok(RuleSetPolicy::BosLifeForceRemoteDns),
            "DIRECT" => Ok(RuleSetPolicy::Direct),
            "DIRECT,no-resolve" => Ok(RuleSetPolicy::DirectNoResolve),
            "DIRECT,force-remote" => Ok(RuleSetPolicy::DirectForceRemoteDns),
            _ => Err(eyre!("无法解析的规则集策略: {}", s)),
        }
    }
}

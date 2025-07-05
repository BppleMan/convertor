use crate::profile::core::policy::Policy;
use crate::profile::error::ParseError;
use crate::profile::renderer::clash_renderer::ClashRenderer;
use crate::profile::renderer::surge_renderer::SurgeRenderer;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone)]
pub struct Rule {
    pub rule_type: RuleType,
    /// 对于 FINAL 和 MATCH 类型的规则，value 是 None
    pub value: Option<String>,
    pub policy: Policy,
    pub comment: Option<String>,
}

impl Rule {
    pub fn surge_rule_provider(policy: &Policy, url: Url) -> Self {
        Self {
            rule_type: RuleType::RuleSet,
            value: Some(url.to_string()),
            policy: policy.clone(),
            comment: Some(SurgeRenderer::render_policy_for_provider(policy)),
        }
    }

    pub fn clash_rule_provider(policy: &Policy) -> Self {
        Self {
            rule_type: RuleType::RuleSet,
            value: Some(ClashRenderer::render_policy_for_provider(policy)),
            policy: policy.clone(),
            comment: None,
        }
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RuleType {
    #[serde(rename = "DOMAIN")]
    Domain,
    #[serde(rename = "DOMAIN-SUFFIX")]
    DomainSuffix,
    #[serde(rename = "DOMAIN-KEYWORD")]
    DomainKeyword,
    #[serde(rename = "PROCESS-NAME")]
    ProcessName,
    #[serde(rename = "USER-AGENT")]
    UserAgent,
    #[serde(rename = "RULE-SET")]
    RuleSet,
    #[serde(rename = "GEOIP")]
    GeoIP,
    #[serde(rename = "IP-CIDR")]
    IpCIDR,
    #[serde(rename = "IP-CIDR6")]
    IpCIDR6,
    #[serde(rename = "FINAL")]
    Final,
    #[serde(rename = "MATCH")]
    Match,
}

impl RuleType {
    pub fn as_str(&self) -> &str {
        match self {
            RuleType::Domain => "DOMAIN",
            RuleType::DomainSuffix => "DOMAIN-SUFFIX",
            RuleType::DomainKeyword => "DOMAIN-KEYWORD",
            RuleType::ProcessName => "PROCESS-NAME",
            RuleType::UserAgent => "USER-AGENT",
            RuleType::RuleSet => "RULE-SET",
            RuleType::GeoIP => "GEOIP",
            RuleType::IpCIDR => "IP-CIDR",
            RuleType::IpCIDR6 => "IP-CIDR6",
            RuleType::Final => "FINAL",
            RuleType::Match => "MATCH",
        }
    }
}

impl Display for RuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RuleType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DOMAIN" => Ok(RuleType::Domain),
            "DOMAIN-SUFFIX" => Ok(RuleType::DomainSuffix),
            "DOMAIN-KEYWORD" => Ok(RuleType::DomainKeyword),
            "PROCESS-NAME" => Ok(RuleType::ProcessName),
            "USER-AGENT" => Ok(RuleType::UserAgent),
            "RULE-SET" => Ok(RuleType::RuleSet),
            "IP-CIDR" => Ok(RuleType::IpCIDR),
            "IP-CIDR6" => Ok(RuleType::IpCIDR6),
            "GEOIP" => Ok(RuleType::GeoIP),
            "FINAL" => Ok(RuleType::Final),
            "MATCH" => Ok(RuleType::Match),
            _ => Err(ParseError::RuleType {
                line: 0,
                reason: format!("未知的规则类型: {}", s),
            }),
        }
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RuleVisitor;

        impl<'de> serde::de::Visitor<'de> for RuleVisitor {
            type Value = Rule;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "规则语法应该形如: 规则类型[,值],策略名称[,选项]")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let rule_parts = v.splitn(3, ',').map(str::trim).collect::<Vec<_>>();

                if rule_parts.len() < 2 {
                    return Err(E::custom("规则语法应该形如: 规则类型[,值],策略名称[,选项]"));
                }

                let rule_type = RuleType::from_str(rule_parts[0]).map_err(E::custom)?;

                let (value, policy) = if rule_parts.len() == 2 {
                    (
                        None,
                        Policy::deserialize(serde::de::value::StrDeserializer::new(rule_parts[1]))?,
                    )
                } else {
                    (
                        Some(rule_parts[1].to_string()),
                        Policy::deserialize(serde::de::value::StrDeserializer::new(rule_parts[2]))?,
                    )
                };

                Ok(Rule {
                    rule_type,
                    value,
                    policy,
                    comment: None,
                })
            }
        }

        deserializer.deserialize_str(RuleVisitor)
    }
}

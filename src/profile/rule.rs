use crate::profile::rule_set_policy::RuleSetPolicy;
use color_eyre::eyre::eyre;
use color_eyre::Report;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Rule {
    pub rule_type: RuleType,
    /// 对于 FINAL 和 MATCH 类型的规则，value 是 None
    pub value: Option<String>,
    pub policy: String,
}

impl Rule {
    pub fn new_rule_set(rsp: RuleSetPolicy) -> Self {
        Self {
            rule_type: RuleType::RuleSet,
            value: Some(rsp.provider_name().to_string()),
            policy: rsp.as_policies().to_string(),
        }
    }

    pub fn serialize(&self) -> String {
        let rule_type = self.rule_type.to_string();
        let mut rule = vec![rule_type];
        if let Some(value) = &self.value {
            rule.push(value.to_string());
        }
        rule.push(self.policy.clone());
        format!(r#"- "{}""#, rule.join(","))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    #[serde(rename = "DOMAIN")]
    Domain,
    #[serde(rename = "DOMAIN-SUFFIX")]
    DomainSuffix,
    #[serde(rename = "DOMAIN-KEYWORD")]
    DomainKeyword,
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
        write!(f, "{}", self.as_str().to_string())
    }
}

impl FromStr for RuleType {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DOMAIN" => Ok(RuleType::Domain),
            "DOMAIN-SUFFIX" => Ok(RuleType::DomainSuffix),
            "DOMAIN-KEYWORD" => Ok(RuleType::DomainKeyword),
            "RULE-SET" => Ok(RuleType::RuleSet),
            "GEOIP" => Ok(RuleType::GeoIP),
            "IP-CIDR" => Ok(RuleType::IpCIDR),
            "IP-CIDR6" => Ok(RuleType::IpCIDR6),
            "FINAL" => Ok(RuleType::Final),
            "MATCH" => Ok(RuleType::Match),
            _ => Err(eyre!("Unknown rule type: {}", s)),
        }
    }
}

impl Serialize for Rule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rule_type = self.rule_type.to_string();
        let mut rule = vec![rule_type];
        if let Some(value) = &self.value {
            rule.push(value.to_string());
        }
        rule.push(self.policy.clone());
        serializer.serialize_str(&rule.join(","))
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

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a comma-separated rule string like RULE_TYPE,value,policy1,..."
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let rule_parts = v.splitn(3, ',').map(str::trim).collect::<Vec<_>>();

                if rule_parts.len() < 2 {
                    return Err(E::custom("rule must have at least 2 parts: rule_type and policy"));
                }

                let rule_type = RuleType::from_str(rule_parts[0]).map_err(E::custom)?;

                let (value, policy) = if rule_parts.len() == 2 {
                    (None, rule_parts[1].to_string())
                } else {
                    (Some(rule_parts[1].to_string()), rule_parts[2].to_string())
                };

                Ok(Rule {
                    rule_type,
                    value,
                    policy,
                })
            }
        }

        deserializer.deserialize_str(RuleVisitor)
    }
}

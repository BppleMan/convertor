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
    pub policies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    #[serde(rename = "DOMAIN")]
    Domain,
    #[serde(rename = "DOMAIN-SUFFIX")]
    DomainSuffix,
    #[serde(rename = "DOMAIN-KEYWORD")]
    DomainKeyword,
    #[serde(rename = "GEOIP")]
    GeoIP,
    #[serde(rename = "IP-CIDR")]
    IpCIDR,
    #[serde(rename = "FINAL")]
    Final,
    #[serde(rename = "MATCH")]
    Match,
}

impl Display for RuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleType::Domain => write!(f, "DOMAIN"),
            RuleType::DomainSuffix => write!(f, "DOMAIN-SUFFIX"),
            RuleType::DomainKeyword => write!(f, "DOMAIN-KEYWORD"),
            RuleType::GeoIP => write!(f, "GEOIP"),
            RuleType::IpCIDR => write!(f, "IP-CIDR"),
            RuleType::Final => write!(f, "FINAL"),
            RuleType::Match => write!(f, "MATCH"),
        }
    }
}

impl FromStr for RuleType {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DOMAIN" => Ok(RuleType::Domain),
            "DOMAIN-SUFFIX" => Ok(RuleType::DomainSuffix),
            "DOMAIN-KEYWORD" => Ok(RuleType::DomainKeyword),
            "GEOIP" => Ok(RuleType::GeoIP),
            "IP-CIDR" => Ok(RuleType::IpCIDR),
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
        rule.push(self.policies.join(","));
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

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(formatter, "a comma-separated rule string like RULE_TYPE,value,policy1,...")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let parts = v.split(',').map(str::trim).collect::<Vec<_>>();

                if parts.len() <= 1 {
                    return Err(E::custom(
                        "rule must have at least 2 parts: rule_type and policy",
                    ));
                }

                let rule_type =
                    RuleType::from_str(parts[0]).map_err(E::custom)?;

                let (value, policies) = if parts.len() == 2 {
                    (None, vec![parts[1].to_string()])
                } else {
                    let value = parts[1].to_string();
                    let policies =
                        parts[2..].iter().map(|s| s.to_string()).collect();
                    (Some(value), policies)
                };

                Ok(Rule {
                    rule_type,
                    value,
                    policies,
                })
            }
        }

        deserializer.deserialize_str(RuleVisitor)
    }
}

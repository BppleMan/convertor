use color_eyre::eyre::eyre;
use color_eyre::Report;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Policy {
    pub name: String,
    pub option: Option<String>,
    pub is_subscription: bool,
}

impl Policy {
    pub fn subscription_policy() -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: None,
            is_subscription: true,
        }
    }

    pub fn is_subscription_policy(&self) -> bool {
        self.is_subscription && self.name == "DIRECT"
    }

    pub fn is_built_in(&self) -> bool {
        (self.name == "DIRECT" || self.name == "REJECT" || self.name == "FINAL") && !self.is_subscription
    }
}

impl FromStr for Policy {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(2, ',').map(str::trim).collect::<Vec<_>>();
        if parts.is_empty() {
            return Err(eyre!("无法解析策略: {}", s));
        }
        Ok(Policy {
            name: parts[0].to_string(),
            option: parts.get(1).map(|part| part.to_string()),
            is_subscription: false,
        })
    }
}

impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PolicyVisitor;

        impl<'de> serde::de::Visitor<'de> for PolicyVisitor {
            type Value = Policy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "策略语法应该形如: 策略名称[,选项]")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Policy::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(PolicyVisitor)
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct QueryPolicy {
    pub name: String,
    pub option: Option<String>,
    pub is_subscription: bool,
}

impl From<Policy> for QueryPolicy {
    fn from(policy: Policy) -> Self {
        QueryPolicy {
            name: policy.name,
            option: policy.option,
            is_subscription: policy.is_subscription,
        }
    }
}

impl From<QueryPolicy> for Policy {
    fn from(query_policy: QueryPolicy) -> Self {
        Policy {
            name: query_policy.name,
            option: query_policy.option,
            is_subscription: query_policy.is_subscription,
        }
    }
}

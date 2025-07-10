use color_eyre::Report;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
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

    pub fn direct_policy() -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: None,
            is_subscription: false,
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

impl PartialOrd<Policy> for Policy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Policy {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then(self.option.cmp(&other.option))
            .then(self.is_subscription.cmp(&other.is_subscription))
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

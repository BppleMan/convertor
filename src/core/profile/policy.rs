use color_eyre::Report;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

static OPTION_RANK: LazyLock<HashMap<Option<&str>, usize>> = LazyLock::new(|| {
    [None, Some("no-resolve"), Some("force-remote-dns")]
        .into_iter()
        .enumerate()
        .map(|(i, option)| (option, i))
        .collect()
});

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Policy {
    pub name: String,
    pub option: Option<String>,
    pub is_subscription: bool,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SerializablePolicy {
    pub name: String,
    pub option: Option<String>,
    pub is_subscription: bool,
}

impl Policy {
    pub fn new(name: impl AsRef<str>, option: Option<&str>, is_subscription: bool) -> Self {
        Policy {
            name: name.as_ref().to_string(),
            option: option.map(|s| s.to_string()),
            is_subscription,
        }
    }

    pub fn subscription_policy() -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: None,
            is_subscription: true,
        }
    }

    pub fn direct_policy(option: Option<&str>) -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: option.map(|s| s.to_string()),
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
        let option_rank = |option: &Option<String>| {
            *OPTION_RANK
                .get(&option.as_ref().map(String::as_str))
                .unwrap_or(&usize::MAX)
        };

        self.is_subscription
            .cmp(&other.is_subscription)
            .reverse()
            .then(self.name.cmp(&other.name))
            .then(option_rank(&self.option).cmp(&option_rank(&other.option)))
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

impl From<Policy> for SerializablePolicy {
    fn from(value: Policy) -> Self {
        SerializablePolicy {
            name: value.name,
            option: value.option,
            is_subscription: value.is_subscription,
        }
    }
}

impl From<&Policy> for SerializablePolicy {
    fn from(value: &Policy) -> Self {
        SerializablePolicy {
            name: value.name.clone(),
            option: value.option.clone(),
            is_subscription: value.is_subscription,
        }
    }
}

impl From<SerializablePolicy> for Policy {
    fn from(value: SerializablePolicy) -> Self {
        Policy {
            name: value.name,
            option: value.option,
            is_subscription: value.is_subscription,
        }
    }
}

impl From<&SerializablePolicy> for Policy {
    fn from(value: &SerializablePolicy) -> Self {
        Policy {
            name: value.name.clone(),
            option: value.option.clone(),
            is_subscription: value.is_subscription,
        }
    }
}

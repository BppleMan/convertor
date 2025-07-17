use crate::client::Client;
use crate::core::profile::policy::Policy;
use crate::encrypt::{decrypt, encrypt};
use color_eyre::Report;
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr, eyre};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileQuery {
    pub client: Client,
    pub original_host: String,
    pub raw_sub_url: String,
    pub interval: u64,
    pub strict: bool,
    pub policy: Option<QueryPolicy>,
}

impl ProfileQuery {
    pub fn encode_to_query_string(&self) -> String {
        let interval_str = self.interval.to_string();
        let mut query_pairs = vec![
            ("client", self.client.as_str()),
            ("original_host", &self.original_host),
            ("interval", &interval_str),
            ("strict", if self.strict { "true" } else { "false" }),
        ];
        if let Some(policy) = &self.policy {
            query_pairs.push(("policy.name", &policy.name));
            if let Some(option) = &policy.option {
                query_pairs.push(("policy.option", option));
            }
            query_pairs.push((
                "policy.is_subscription",
                if policy.is_subscription { "true" } else { "false" },
            ));
        }
        query_pairs.push(("raw_sub_url", &self.raw_sub_url));

        query_pairs
            .into_iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k, percent_encoding::CONTROLS),
                    utf8_percent_encode(v, percent_encoding::CONTROLS)
                )
            })
            .collect::<Vec<_>>()
            .join("&")
    }

    pub fn decode_from_query_string(query_string: impl AsRef<str>) -> Result<ProfileQuery> {
        let query_pairs = query_string
            .as_ref()
            .split('&')
            .filter_map(|p| p.split_once('='))
            .map(|(k, v)| {
                Ok::<_, Report>(percent_decode_str(k.trim()).decode_utf8()?)
                    .and_then(|k| Ok::<_, Report>(percent_decode_str(v.trim()).decode_utf8()?).map(|v| (k, v)))
            })
            .collect::<Result<HashMap<Cow<'_, str>, Cow<'_, str>>, Report>>()?;
        let client: Client = query_pairs.get("client").ok_or_eyre("缺少 client 参数")?.parse()?;
        let original_host = query_pairs
            .get("original_host")
            .ok_or_eyre("缺少 original_host 参数")?
            .to_string();
        let raw_sub_url = query_pairs
            .get("raw_sub_url")
            .ok_or_eyre("缺少 raw_sub_url 参数")?
            .to_string();
        let interval = query_pairs
            .get("interval")
            .map(|s| s.parse::<u64>())
            .transpose()
            .map_err(|e| eyre!("interval 不是一个合法的 u64: {}", e))?
            .unwrap_or(86400);
        let strict = query_pairs
            .get("strict")
            .map(|s| s.parse::<bool>())
            .transpose()
            .map_err(|e| eyre!("strict 不是一个合法的 bool: {}", e))?
            .unwrap_or(true);
        let policy_name = query_pairs.get("policy.name");
        let policy_option = query_pairs.get("policy.option");
        let is_subscription = query_pairs
            .get("policy.is_subscription")
            .map(|s| s.parse::<bool>())
            .transpose()
            .wrap_err_with(|| "policy.is_subscription 不是一个合法的 bool")?;
        let policy = match (policy_name, is_subscription) {
            (Some(name), Some(is_subscription)) => Some(QueryPolicy {
                name: name.to_string(),
                option: policy_option.map(Cow::<str>::to_string),
                is_subscription,
            }),
            _ => None,
        };

        Ok(ProfileQuery {
            client,
            original_host,
            raw_sub_url,
            interval,
            strict,
            policy,
        })
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SubLogQuery {
    pub secret: String,
    pub page_current: Option<usize>,
    pub page_size: Option<usize>,
}

impl SubLogQuery {
    pub fn new(secret: impl AsRef<str>, page_current: Option<usize>, page_size: Option<usize>) -> Self {
        SubLogQuery {
            secret: secret.as_ref().to_string(),
            page_current,
            page_size,
        }
    }

    pub fn encode_to_query_string(&self) -> Result<String> {
        let mut query_pairs = vec![];
        let encrypted_secret = encrypt(self.secret.as_bytes(), self.secret.as_str())?;
        query_pairs.push(format!(
            "secret={}",
            utf8_percent_encode(&encrypted_secret, percent_encoding::CONTROLS)
        ));
        if let Some(page_current) = self.page_current {
            query_pairs.push(format!("page_current={}", page_current));
        }
        if let Some(page_size) = self.page_size {
            query_pairs.push(format!("page_size={}", page_size));
        }
        query_pairs.sort();
        Ok(query_pairs.join("&"))
    }

    pub fn decode_from_query_string(query_string: impl AsRef<str>, secret: impl AsRef<str>) -> Result<SubLogQuery> {
        let query_pairs = query_string
            .as_ref()
            .split('&')
            .filter_map(|p| p.split_once('='))
            .collect::<HashMap<_, _>>();
        let secret = query_pairs
            .get("secret")
            .map(|s| percent_decode_str(s).decode_utf8())
            .transpose()
            .wrap_err("无法进行 url decoding")?
            .map(|s| decrypt(secret.as_ref().as_bytes(), &*s))
            .transpose()
            .wrap_err("未认证的 secret")?
            .ok_or_eyre("缺少 secret 参数")?
            .to_string();
        let page_current = query_pairs.get("page_current").and_then(|s| s.parse::<usize>().ok());
        let page_size = query_pairs.get("page_size").and_then(|s| s.parse::<usize>().ok());

        Ok(SubLogQuery {
            secret,
            page_current,
            page_size,
        })
    }
}

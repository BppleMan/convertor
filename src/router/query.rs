use crate::client::Client;
use crate::profile::core::policy::Policy;
use color_eyre::eyre::WrapErr;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileQuery {
    pub client: Client,
    pub original_host: String,
    pub raw_sub_url: String,
    pub policy: Option<QueryPolicy>,
}

impl ProfileQuery {
    pub fn decode_from_query_string(raw_query_string: impl AsRef<str>) -> color_eyre::Result<ProfileQuery> {
        let query_string = percent_decode_str(raw_query_string.as_ref())
            .decode_utf8()
            .wrap_err("无法解码查询字符串")?
            .to_string();
        println!("query_string: {}", query_string);
        Ok(serde_qs::from_str(&query_string)?)
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

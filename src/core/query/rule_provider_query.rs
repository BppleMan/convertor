use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::sub_provider::SubProvider;
use crate::common::encrypt::decrypt;
use crate::core::profile::policy::Policy;
use crate::core::query::error::{ConvertorQueryError, ParseError};
use crate::server::ProfileCacheKey;
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::Utf8Error;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProviderQuery {
    pub client: ProxyClient,
    pub provider: SubProvider,
    pub server: Url,
    pub uni_sub_url: Url,
    pub enc_uni_sub_url: String,
    pub interval: u64,
    pub policy: SerializablePolicy,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SerializablePolicy {
    pub name: String,
    pub option: Option<String>,
    pub is_subscription: bool,
}

impl RuleProviderQuery {
    pub fn parse_from_query_string(
        query_string: impl AsRef<str>,
        secret: impl AsRef<str>,
    ) -> Result<Self, ConvertorQueryError> {
        let query_string = query_string.as_ref();
        let secret = secret.as_ref().to_string();
        let query_map = query_string
            .split('&')
            .filter_map(|p| p.split_once('='))
            .map(|(k, v)| {
                percent_decode_str(k.trim())
                    .decode_utf8()
                    .and_then(|k| percent_decode_str(v.trim()).decode_utf8().map(|v| (k, v)))
            })
            .collect::<Result<HashMap<Cow<'_, str>, Cow<'_, str>>, Utf8Error>>()
            .map_err(ParseError::from)?;

        // 解析 client
        let client = query_map
            .get("client")
            .ok_or(ParseError::NotFoundParam("client"))?
            .parse::<ProxyClient>()
            .map_err(ParseError::from)?;

        // 解析 provider
        let provider = query_map
            .get("provider")
            .ok_or(ParseError::NotFoundParam("provider"))?
            .parse::<SubProvider>()
            .map_err(ParseError::from)?;

        // 解析 server
        let server = query_map
            .get("server")
            .ok_or(ParseError::NotFoundParam("server"))?
            .parse::<Url>()
            .map_err(ParseError::from)?;

        // 解析 uni_sub_url
        let enc_uni_sub_url = query_map
            .get("uni_sub_url")
            .ok_or(ParseError::NotFoundParam("uni_sub_url"))?
            .to_string();
        let uni_sub_url = decrypt(secret.as_bytes(), enc_uni_sub_url.as_ref())?
            .parse::<Url>()
            .map_err(ParseError::from)?;

        // 解析 interval
        let interval = query_map
            .get("interval")
            .map(|s| s.parse::<u64>())
            .transpose()
            .map_err(ParseError::from)?
            .unwrap_or(86400);

        // 解析 policy
        let policy = SerializablePolicy::parse_from_query_pairs(&query_map)?;

        let query = RuleProviderQuery {
            client,
            provider,
            server,
            uni_sub_url,
            enc_uni_sub_url,
            interval,
            policy,
        };
        Ok(query)
    }

    pub fn encode_to_query_string(&self) -> String {
        let mut query_pairs = vec![
            ("client", Cow::Borrowed(self.client.as_str())),
            ("provider", Cow::Borrowed(self.provider.as_str())),
            ("server", Cow::Borrowed(self.server.as_str())),
            ("interval", Cow::Owned(self.interval.to_string())),
        ];
        self.policy.encode_to_query_pairs(&mut query_pairs);
        query_pairs.push(("uni_sub_url", Cow::Borrowed(&self.enc_uni_sub_url)));

        query_pairs
            .into_iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k, percent_encoding::CONTROLS),
                    utf8_percent_encode(v.as_ref(), percent_encoding::CONTROLS)
                )
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

impl RuleProviderQuery {
    pub fn cache_key(&self) -> ProfileCacheKey {
        ProfileCacheKey {
            client: self.client,
            provider: self.provider,
            uni_sub_url: self.uni_sub_url.clone(),
            interval: self.interval,
            server: None,
            strict: None,
            policy: Some(self.policy.clone().into()),
        }
    }

    pub fn encoded_uni_sub_url(&self) -> String {
        utf8_percent_encode(&self.enc_uni_sub_url, percent_encoding::CONTROLS).to_string()
    }
}

impl SerializablePolicy {
    pub fn parse_from_query_pairs(query_pairs: &HashMap<Cow<'_, str>, Cow<'_, str>>) -> Result<Self, ParseError> {
        let name = query_pairs
            .get("policy.name")
            .ok_or(ParseError::NotFoundParam("policy.name"))?
            .to_string();
        let option = query_pairs.get("policy.option").map(|s| s.to_string());
        let is_subscription = query_pairs
            .get("policy.is_subscription")
            .map(|s| s.parse::<bool>())
            .transpose()
            .map_err(ParseError::from)?
            .unwrap_or(false);

        Ok(SerializablePolicy {
            name,
            option,
            is_subscription,
        })
    }

    pub fn encode_to_query_pairs<'a, 'b>(&'a self, query_pairs: &mut Vec<(&'static str, Cow<'b, str>)>)
    where
        'a: 'b,
    {
        query_pairs.push(("policy.name", Cow::Borrowed(&self.name)));
        if let Some(option) = &self.option {
            query_pairs.push(("policy.option", Cow::Borrowed(option)));
        }
        query_pairs.push(("policy.is_subscription", Cow::Owned(self.is_subscription.to_string())));
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

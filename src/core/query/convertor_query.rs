use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::sub_provider::SubProvider;
use crate::common::encrypt::decrypt;
use crate::core::profile::policy::SerializablePolicy;
use crate::core::query::error::{ConvertorQueryError, ParseError};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::Utf8Error;
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ConvertorQuery {
    pub client: ProxyClient,
    pub provider: SubProvider,
    pub server: Url,
    /// 通用订阅地址
    pub uni_sub_url: Url,
    /// 加密后的通用订阅地址
    pub enc_uni_sub_url: String,
    pub interval: u64,
    pub strict: bool,
    pub policy: Option<SerializablePolicy>,
}

impl ConvertorQuery {
    pub fn set_policy(mut self, policy: Option<SerializablePolicy>) -> Self {
        self.policy = policy;
        self
    }

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

        // 解析 strict
        let strict = query_map
            .get("strict")
            .map(|s| s.parse::<bool>())
            .transpose()
            .map_err(ParseError::from)?
            .unwrap_or(true);

        // 解析 policy
        let policy_name = query_map.get("policy.name");
        let policy_option = query_map.get("policy.option");
        let is_subscription = query_map
            .get("policy.is_subscription")
            .map(|s| s.parse::<bool>())
            .transpose()
            .map_err(ParseError::from)?;

        let policy = match (policy_name, is_subscription) {
            (Some(name), Some(is_subscription)) => Some(SerializablePolicy {
                name: name.to_string(),
                option: policy_option.map(Cow::<str>::to_string),
                is_subscription,
            }),
            _ => None,
        };

        Ok(Self {
            client,
            provider,
            server,
            uni_sub_url,
            enc_uni_sub_url,
            interval,
            strict,
            policy,
        })
    }

    pub fn encode_to_query_string(&self) -> String {
        let mut query_pairs = vec![
            ("client", Cow::Borrowed(self.client.as_str())),
            ("provider", Cow::Borrowed(self.provider.as_str())),
            ("server", Cow::Borrowed(self.server.as_str())),
            ("interval", Cow::Owned(self.interval.to_string())),
            ("strict", Cow::Owned(self.strict.to_string())),
        ];
        if let Some(policy) = &self.policy {
            query_pairs.push(("policy.name", Cow::Borrowed(&policy.name)));
            if let Some(option) = &policy.option {
                query_pairs.push(("policy.option", Cow::Borrowed(option)));
            }
            query_pairs.push(("policy.is_subscription", Cow::Owned(policy.is_subscription.to_string())));
        }
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

    pub fn encoded_uni_sub_url(&self) -> String {
        utf8_percent_encode(&self.enc_uni_sub_url, percent_encoding::CONTROLS).to_string()
    }
}

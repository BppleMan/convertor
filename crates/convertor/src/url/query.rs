use crate::common::config::provider_config::Provider;
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::encrypt::decrypt;
use crate::core::profile::policy::Policy;
use crate::url::url_error::{EncodeError, ParseError, QueryError};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::Utf8Error;
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConvertorQuery {
    // common
    pub server: Url,
    pub client: ProxyClient,
    pub provider: Provider,
    pub sub_url: Url,
    pub enc_sub_url: String,
    pub interval: u64,

    // profile
    pub strict: Option<bool>,

    // rule provider
    pub policy: Option<Policy>,

    // sub logs
    pub secret: Option<String>,
    pub enc_secret: Option<String>,
}

impl ConvertorQuery {
    pub fn parse_from_query_string(
        query_string: impl AsRef<str>,
        secret: impl AsRef<str>,
        server: Url,
        client: ProxyClient,
        provider: Provider,
    ) -> Result<Self, QueryError> {
        let query_string = query_string.as_ref();
        let secret = secret.as_ref();
        let query_map = Self::url_decode(query_string)?;

        // 解析 sub_url
        let enc_sub_url = query_map
            .get("sub_url")
            .ok_or(ParseError::NotFoundParam("sub_url"))?
            .to_string();
        let sub_url = decrypt(secret.as_bytes(), enc_sub_url.as_ref())?
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
            .map_err(ParseError::from)?;

        // 解析 policy
        let policy = Self::parse_policy_from_query_pairs(&query_map)?;

        // 解析 secret
        let enc_secret = query_map
            .get("secret")
            .map(|s| {
                percent_decode_str(s.as_ref())
                    .decode_utf8()
                    .map(|es| es.to_string())
                    .map_err(ParseError::from)
            })
            .transpose()?;
        let secret = enc_secret
            .as_ref()
            .map(|es| decrypt(secret.as_bytes(), es.as_ref()))
            .transpose()?;

        Ok(Self {
            server,
            client,
            provider,
            sub_url,
            enc_sub_url,
            interval,
            strict,
            policy,
            secret,
            enc_secret,
        })
    }

    fn parse_policy_from_query_pairs(
        query_map: &HashMap<Cow<'_, str>, Cow<'_, str>>,
    ) -> Result<Option<Policy>, ParseError> {
        let name = query_map.get("policy[name]").map(|s| s.to_string());
        let option = query_map.get("policy[option]").map(|s| s.to_string());
        let is_subscription = query_map
            .get("policy[is_subscription]")
            .map(|s| s.parse::<bool>())
            .transpose()?;
        let policy = if let (Some(name), option, Some(is_subscription)) = (name, option, is_subscription) {
            Some(Policy {
                name,
                option,
                is_subscription,
            })
        } else {
            None
        };
        Ok(policy)
    }

    pub fn encode_to_profile_query(&self) -> Result<String, QueryError> {
        let interval_str = self.interval.to_string();
        let strict_str = self
            .strict
            .ok_or(EncodeError::NotFoundParam("profile", "strict"))?
            .to_string();
        let query_pairs = vec![
            ("interval", Cow::Borrowed(interval_str.as_str())),
            ("strict", Cow::Borrowed(strict_str.as_str())),
            ("sub_url", Cow::Borrowed(self.enc_sub_url.as_str())),
        ];

        Ok(Self::url_encode(query_pairs))
    }

    pub fn encode_to_rule_provider_query(&self) -> Result<String, QueryError> {
        let policy = self
            .policy
            .as_ref()
            .ok_or(EncodeError::NotFoundParam("rule provider", "policy"))?;
        let mut query_pairs = vec![("interval", Cow::Owned(self.interval.to_string()))];
        Self::encode_policy_to_query_pairs(policy, &mut query_pairs);
        query_pairs.push(("sub_url", Cow::Borrowed(&self.enc_sub_url)));

        Ok(Self::url_encode(query_pairs))
    }

    pub fn encode_to_sub_logs_query(&self) -> Result<String, QueryError> {
        let enc_secret = self
            .enc_secret
            .as_ref()
            .ok_or(EncodeError::NotFoundParam("sub logs", "enc_secret"))?;

        let query_pairs = vec![("secret", Cow::Owned(enc_secret.clone()))];

        Ok(Self::url_encode(query_pairs))
    }

    fn encode_policy_to_query_pairs(policy: &Policy, query_pairs: &mut Vec<(&str, Cow<'_, str>)>) {
        query_pairs.push(("policy[name]", Cow::Owned(policy.name.clone())));
        if let Some(option) = &policy.option {
            query_pairs.push(("policy[option]", Cow::Owned(option.clone())));
        }
        query_pairs.push((
            "policy[is_subscription]",
            Cow::Owned(policy.is_subscription.to_string()),
        ));
    }

    pub fn encoded_sub_url(&self) -> String {
        utf8_percent_encode(&self.enc_sub_url, percent_encoding::CONTROLS).to_string()
    }
}

impl ConvertorQuery {
    fn url_decode(query_string: &str) -> Result<HashMap<Cow<'_, str>, Cow<'_, str>>, ParseError> {
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
        Ok(query_map)
    }

    fn url_encode<'a>(query_pairs: impl IntoIterator<Item = (&'static str, Cow<'a, str>)>) -> String {
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

impl ConvertorQuery {
    pub fn check_for_profile(&self) -> Result<(), QueryError> {
        if self.strict.is_none() {
            return Err(QueryError::Encode(EncodeError::NotFoundParam("profile", "strict")));
        }
        Ok(())
    }

    pub fn check_for_rule_provider(&self) -> Result<(), QueryError> {
        if self.policy.is_none() {
            return Err(QueryError::Encode(EncodeError::NotFoundParam(
                "rule provider",
                "policy",
            )));
        }
        Ok(())
    }

    pub fn check_for_sub_logs(&self) -> Result<(), QueryError> {
        if self.secret.is_none() {
            return Err(QueryError::Encode(EncodeError::NotFoundParam("sub logs", "secret")));
        }
        if self.enc_secret.is_none() {
            return Err(QueryError::Encode(EncodeError::NotFoundParam("sub logs", "enc_secret")));
        }
        Ok(())
    }
}

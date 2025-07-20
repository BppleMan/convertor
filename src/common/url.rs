use crate::common::config::proxy_client::{ParseClientError, ProxyClient};
use crate::common::encrypt::{EncryptError, decrypt, encrypt};
use crate::core::profile::policy::{Policy, SerializablePolicy};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::Utf8Error;
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ConvertorUrl {
    pub secret: String,
    pub client: ProxyClient,
    pub server: Url,
    /// 通用订阅地址
    pub uni_sub_url: Url,
    /// 加密后的通用订阅地址
    pub enc_uni_sub_url: String,
    pub interval: u64,
    pub strict: bool,
    pub policy: Option<SerializablePolicy>,
}

impl ConvertorUrl {
    pub fn new(
        secret: impl AsRef<str>,
        client: ProxyClient,
        server: Url,
        uni_sub_url: Url,
        interval: u64,
        strict: bool,
        policy: Option<SerializablePolicy>,
    ) -> Result<Self, ConvertorUrlError> {
        let secret = secret.as_ref().to_string();
        let enc_uni_sub_url = encrypt(secret.as_bytes(), uni_sub_url.as_str())?;
        let url = Self {
            secret,
            client,
            server,
            uni_sub_url,
            enc_uni_sub_url,
            interval,
            strict,
            policy,
        };
        Ok(url)
    }

    pub fn uni_sub_host(&self) -> Result<String, ConvertorUrlError> {
        match (self.uni_sub_url.host_str(), self.uni_sub_url.port()) {
            (Some(host), Some(port)) => Ok(format!("{host}:{port}")),
            (Some(host), None) => Ok(host.to_string()),
            _ => Err(ConvertorUrlError::Encode(EncodeError::NoRawSubHost(
                self.uni_sub_url.to_string(),
            ))),
        }
    }

    pub fn parse_from_url(url: &Url, secret: impl AsRef<str>) -> Result<Self, ConvertorUrlError> {
        let secret = secret.as_ref();
        let Some(query) = url.query() else {
            return Err(ConvertorUrlError::Parse(ParseError::UrlNoQuery(url.to_string())));
        };
        Self::parse_from_query_string(query, secret)
    }

    /// 构造原始订阅地址，主要是用通用订阅地址拼接客户端标志
    pub fn build_raw_sub_url(&self) -> Result<Url, ConvertorUrlError> {
        let mut url = self.uni_sub_url.clone();
        url.query_pairs_mut().append_pair("flag", self.client.as_str());
        Ok(url)
    }

    /// 构造编码后的通用订阅地址，编码前会先加密
    pub fn encoded_uni_sub_url(&self) -> Result<String, ConvertorUrlError> {
        let raw_sub_url = utf8_percent_encode(&self.enc_uni_sub_url, percent_encoding::CONTROLS).to_string();
        Ok(raw_sub_url)
    }

    pub fn build_sub_url(&self) -> Result<Url, ConvertorUrlError> {
        let query_string = self.encode_to_query_string(None)?;
        let mut url = self.server.clone();
        url.set_path("/profile");
        url.set_query(Some(&query_string));
        Ok(url)
    }

    pub fn build_rule_provider_url(&self, policy: &Policy) -> Result<Url, ConvertorUrlError> {
        let query_string = self.encode_to_query_string(Some(policy))?;
        let mut url = self.server.clone();
        url.set_path("/rule-provider");
        url.set_query(Some(&query_string));
        Ok(url)
    }

    pub fn build_managed_config_header(&self, for_raw: bool) -> Result<String, ConvertorUrlError> {
        let sub_url = if for_raw {
            self.build_raw_sub_url()
        } else {
            self.build_sub_url()
        }?;
        let header = format!(
            "#!MANAGED-CONFIG {} interval={} strict={}",
            sub_url, self.interval, self.strict
        );
        Ok(header)
    }

    pub fn build_sub_logs_url(&self, sub_log_query: impl AsRef<str>) -> Result<Url, ConvertorUrlError> {
        let mut url = self.server.clone();
        url.set_path("/sub-logs");
        url.set_query(Some(sub_log_query.as_ref()));
        Ok(url)
    }

    pub fn parse_from_query_string(
        query_string: impl AsRef<str>,
        secret: impl AsRef<str>,
    ) -> Result<Self, ConvertorUrlError> {
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
            secret,
            client,
            server,
            uni_sub_url,
            enc_uni_sub_url,
            interval,
            strict,
            policy,
        })
    }

    pub fn encode_to_query_string(&self, policy: Option<&Policy>) -> Result<String, ConvertorUrlError> {
        let mut query_pairs = vec![
            ("client", Cow::Borrowed(self.client.as_str())),
            ("server", Cow::Borrowed(self.server.as_str())),
            ("interval", Cow::Owned(self.interval.to_string())),
            ("strict", Cow::Owned(self.strict.to_string())),
        ];
        if let Some(policy) = policy {
            query_pairs.push(("policy.name", Cow::Borrowed(&policy.name)));
            if let Some(option) = &policy.option {
                query_pairs.push(("policy.option", Cow::Borrowed(option)));
            }
            query_pairs.push(("policy.is_subscription", Cow::Owned(policy.is_subscription.to_string())));
        }
        query_pairs.push(("uni_sub_url", Cow::Borrowed(&self.enc_uni_sub_url)));

        let query_string = query_pairs
            .into_iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k, percent_encoding::CONTROLS),
                    utf8_percent_encode(v.as_ref(), percent_encoding::CONTROLS)
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        Ok(query_string)
    }
}

#[derive(Debug, Error)]
pub enum ConvertorUrlError {
    #[error("无法加密/解密 raw_sub_url: {0}")]
    EncryptError(#[from] EncryptError),

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Encode(#[from] EncodeError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("无法从 URL 中解析 ConvertorUrl: 缺少查询参数: {0}")]
    NotFoundParam(&'static str),

    #[error("无法从 URL 中解析 ConvertorUrl: 没有查询字符串")]
    UrlNoQuery(String),

    #[error(transparent)]
    ParseClientError(#[from] ParseClientError),

    #[error(transparent)]
    ParseServerError(#[from] url::ParseError),

    #[error(transparent)]
    ParseNumError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("构造 rule-provider URL 失败: 必须提供 policy")]
    RuleProviderNoPolicy,

    #[error("无法提取 raw_sub_url 的主机和端口信息: {0}")]
    NoRawSubHost(String),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

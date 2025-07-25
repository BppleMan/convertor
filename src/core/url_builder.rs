use crate::common::config::proxy_client::ProxyClient;
use crate::common::config::sub_provider::SubProvider;
use crate::common::encrypt::{EncryptError, encrypt};
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_header::SurgeHeader;
use crate::core::query::convertor_query::ConvertorQuery;
use crate::core::query::error::ConvertorQueryError;
use crate::core::query::sub_logs_query::SubLogsQuery;
use crate::core::url_builder::convertor_url::ConvertorUrl;
use crate::core::url_builder::raw_sub_url::RawSubUrl;
use crate::core::url_builder::sub_logs_url::SubLogsUrl;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub mod convertor_url;
pub mod raw_sub_url;
pub mod sub_logs_url;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UrlBuilder {
    pub secret: String,
    pub client: ProxyClient,
    pub provider: SubProvider,
    pub server: Url,
    /// 通用订阅地址
    pub uni_sub_url: Url,
    /// 加密后的通用订阅地址
    pub enc_uni_sub_url: String,
    pub interval: u64,
    pub strict: bool,
}

impl UrlBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        secret: impl AsRef<str>,
        client: ProxyClient,
        provider: SubProvider,
        server: Url,
        uni_sub_url: Url,
        enc_uni_sub_url: Option<String>,
        interval: u64,
        strict: bool,
    ) -> Result<Self, EncryptError> {
        let secret = secret.as_ref().to_string();
        let enc_uni_sub_url = enc_uni_sub_url
            .map(Ok)
            .unwrap_or_else(|| encrypt(secret.as_bytes(), uni_sub_url.as_str()))?;

        let url = Self {
            secret,
            client,
            provider,
            server,
            uni_sub_url,
            enc_uni_sub_url,
            interval,
            strict,
        };
        Ok(url)
    }

    pub fn from_query(secret: impl AsRef<str>, query: ConvertorQuery) -> Result<Self, EncryptError> {
        let ConvertorQuery {
            client,
            provider,
            server,
            uni_sub_url,
            enc_uni_sub_url,
            interval,
            strict,
            policy: _,
        } = query;
        Self::new(
            secret,
            client,
            provider,
            server,
            uni_sub_url,
            Some(enc_uni_sub_url),
            interval,
            strict,
        )
    }

    pub fn parse_from_url(url: &Url, secret: impl AsRef<str>) -> Result<Self, ConvertorUrlError> {
        let secret = secret.as_ref();
        match url.query() {
            None => Err(ConvertorUrlError::ParseFromUrlNoQuery(url.clone())),
            Some(query) => {
                let query = ConvertorQuery::parse_from_query_string(query, secret)?;
                Ok(Self::from_query(secret, query)?)
            }
        }
    }

    pub fn build_raw_sub_url(&self) -> Result<RawSubUrl, ConvertorUrlError> {
        let server = self.uni_sub_url.clone();
        let flag = self.client;
        Ok(RawSubUrl { server, flag })
    }

    pub fn build_sub_url(&self) -> Result<ConvertorUrl, ConvertorUrlError> {
        let server = self.server.clone();
        let path = "/profile".to_string();
        let query = self.into();
        Ok(ConvertorUrl { server, path, query })
    }

    pub fn build_rule_provider_url(&self, policy: &Policy) -> Result<ConvertorUrl, ConvertorUrlError> {
        let server = self.server.clone();
        let path = "/rule-provider".to_string();
        let query = ConvertorQuery::from(self).set_policy(Some(policy.into()));
        Ok(ConvertorUrl { server, path, query })
    }

    pub fn build_sub_logs_url(&self, page: usize, page_size: usize) -> Result<SubLogsUrl, ConvertorUrlError> {
        let server = self.server.clone();
        let path = "/sub-logs".to_string();
        let enc_secret = encrypt(self.secret.as_bytes(), &self.secret)?;
        let query = SubLogsQuery::new(self.provider, &self.secret, enc_secret, page, page_size);
        Ok(SubLogsUrl { server, path, query })
    }

    pub fn build_managed_config_header(&self, for_raw: bool) -> Result<SurgeHeader, ConvertorUrlError> {
        let header = if for_raw {
            SurgeHeader::new_raw(self.build_raw_sub_url()?, self.interval, self.strict)
        } else {
            SurgeHeader::new_convertor(self.build_sub_url()?, self.interval, self.strict)
        };
        Ok(header)
    }
}

pub trait HostPort {
    fn host_port(&self) -> Result<String, ConvertorUrlError>;
}

impl HostPort for Url {
    fn host_port(&self) -> Result<String, ConvertorUrlError> {
        match (self.host_str(), self.port()) {
            (Some(host), Some(port)) => Ok(format!("{host}:{port}")),
            (Some(host), None) => Ok(host.to_string()),
            _ => Err(ConvertorUrlError::NoUniSubHost(self.clone())),
        }
    }
}

impl From<&UrlBuilder> for ConvertorQuery {
    fn from(builder: &UrlBuilder) -> Self {
        ConvertorQuery {
            client: builder.client,
            provider: builder.provider,
            server: builder.server.clone(),
            uni_sub_url: builder.uni_sub_url.clone(),
            enc_uni_sub_url: builder.enc_uni_sub_url.clone(),
            interval: builder.interval,
            strict: builder.strict,
            policy: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConvertorUrlError {
    #[error("无法获取 uni_sub_host: {0}")]
    NoUniSubHost(Url),

    #[error("从 URL 中解析失败, 没有 query 参数: {0}")]
    ParseFromUrlNoQuery(Url),

    #[error(transparent)]
    ConvertorQueryError(#[from] ConvertorQueryError),

    #[error("无法加密/解密 raw_sub_url: {0}")]
    EncryptError(#[from] EncryptError),
}

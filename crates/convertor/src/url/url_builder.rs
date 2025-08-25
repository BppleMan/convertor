use crate::common::config::provider_config::Provider;
use crate::common::config::proxy_client_config::ProxyClient;
use crate::common::encrypt::encrypt;
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_header::SurgeHeader;
use crate::url::convertor_url::{ConvertorUrl, ConvertorUrlType};
use crate::url::query::ConvertorQuery;
use crate::url::url_error::UrlBuilderError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UrlBuilder {
    pub secret: String,
    pub enc_secret: String,
    pub client: ProxyClient,
    pub provider: Provider,
    pub server: Url,
    pub sub_url: Url,
    pub enc_sub_url: String,
    pub interval: u64,
    pub strict: bool,
}

impl UrlBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        secret: impl AsRef<str>,
        enc_secret: Option<String>,
        client: ProxyClient,
        provider: Provider,
        server: Url,
        sub_url: Url,
        enc_sub_url: Option<String>,
        interval: u64,
        strict: bool,
    ) -> Result<Self, UrlBuilderError> {
        let secret = secret.as_ref().to_string();
        let enc_secret = enc_secret
            .map(Ok)
            .unwrap_or_else(|| encrypt(secret.as_bytes(), secret.as_str()))?;
        let enc_sub_url = enc_sub_url
            .map(Ok)
            .unwrap_or_else(|| encrypt(secret.as_bytes(), sub_url.as_str()))?;

        let builder = Self {
            secret,
            enc_secret,
            client,
            provider,
            server,
            sub_url,
            enc_sub_url,
            interval,
            strict,
        };
        Ok(builder)
    }

    pub fn from_convertor_query(query: ConvertorQuery, secret: impl AsRef<str>) -> Result<Self, UrlBuilderError> {
        let ConvertorQuery {
            server,
            client,
            provider,
            sub_url,
            enc_sub_url,
            interval,
            strict,
            secret: secret_opt,
            enc_secret,
            policy: _,
        } = query;
        let secret = secret_opt.unwrap_or(secret.as_ref().to_string());
        let strict = strict.unwrap_or(true);
        Self::new(
            secret,
            enc_secret,
            client,
            provider,
            server,
            sub_url,
            Some(enc_sub_url),
            interval,
            strict,
        )
    }

    // 构造直通 raw 订阅链接
    pub fn build_raw_url(&self) -> ConvertorUrl {
        ConvertorUrl {
            r#type: ConvertorUrlType::Raw,
            server: self.sub_url.clone(),
            path: self.sub_url.path().to_string(),
            query: format!("flag={}", self.client),
        }
    }

    // 构造转发 raw 订阅链接
    pub fn build_raw_profile_url(&self) -> Result<ConvertorUrl, UrlBuilderError> {
        let url = ConvertorUrl {
            r#type: ConvertorUrlType::RawProfile,
            server: self.server.clone(),
            path: format!(
                "/raw-profile/{}/{}",
                self.client.as_ref().to_ascii_lowercase(),
                self.provider.as_ref().to_ascii_lowercase()
            ),
            query: self.as_profile_query().encode_to_profile_query()?,
        };
        Ok(url)
    }

    // 构造转发的 profile 链接
    pub fn build_profile_url(&self) -> Result<ConvertorUrl, UrlBuilderError> {
        let url = ConvertorUrl {
            r#type: ConvertorUrlType::Profile,
            server: self.server.clone(),
            path: format!(
                "/profile/{}/{}",
                self.client.as_ref().to_ascii_lowercase(),
                self.provider.as_ref().to_lowercase()
            ),
            query: self.as_profile_query().encode_to_profile_query()?,
        };
        Ok(url)
    }

    // 构造转发的 规则集 链接
    pub fn build_rule_provider_url(&self, policy: &Policy) -> Result<ConvertorUrl, UrlBuilderError> {
        let url = ConvertorUrl {
            r#type: ConvertorUrlType::RuleProvider,
            server: self.server.clone(),
            path: format!(
                "/rule-provider/{}/{}",
                self.client.as_ref().to_ascii_lowercase(),
                self.provider.as_ref().to_ascii_lowercase()
            ),
            query: self.as_rule_provider_query(policy).encode_to_rule_provider_query()?,
        };
        Ok(url)
    }

    // 构造转发的 订阅日志 链接
    pub fn build_sub_logs_url(&self) -> Result<ConvertorUrl, UrlBuilderError> {
        let url = ConvertorUrl {
            r#type: ConvertorUrlType::SubLogs,
            server: self.server.clone(),
            path: "/sub-logs".to_string(),
            query: self.as_sub_logs_query().encode_to_sub_logs_query()?,
        };
        Ok(url)
    }

    // 构造专属 Surge 的订阅头
    pub fn build_surge_header(&self, r#type: ConvertorUrlType) -> Result<SurgeHeader, UrlBuilderError> {
        let url = match r#type {
            ConvertorUrlType::Raw => self.build_raw_url(),
            ConvertorUrlType::RawProfile => self.build_raw_profile_url()?,
            ConvertorUrlType::Profile => self.build_profile_url()?,
            _ => return Err(UrlBuilderError::UnsupportedUrlType(r#type)),
        };
        Ok(SurgeHeader::new(url, self.interval, self.strict))
    }
}

impl UrlBuilder {
    pub fn as_profile_query(&self) -> ConvertorQuery {
        ConvertorQuery {
            server: self.server.clone(),
            client: self.client,
            provider: self.provider,
            sub_url: self.sub_url.clone(),
            enc_sub_url: self.enc_sub_url.clone(),
            interval: self.interval,
            strict: Some(self.strict),
            policy: None,
            secret: None,
            enc_secret: None,
        }
    }

    pub fn as_rule_provider_query(&self, policy: &Policy) -> ConvertorQuery {
        let mut query = self.as_profile_query();
        query.policy = Some(policy.clone());
        query
    }

    pub fn as_sub_logs_query(&self) -> ConvertorQuery {
        let mut query = self.as_profile_query();
        query.secret = Some(self.secret.clone());
        query.enc_secret = Some(self.enc_secret.clone());
        query
    }
}

pub trait HostPort {
    fn host_port(&self) -> Result<String, UrlBuilderError>;
}

impl HostPort for Url {
    fn host_port(&self) -> Result<String, UrlBuilderError> {
        match (self.host_str(), self.port()) {
            (Some(host), Some(port)) => Ok(format!("{host}:{port}")),
            (Some(host), None) => Ok(host.to_string()),
            _ => Err(UrlBuilderError::NoUniSubHost(self.clone())),
        }
    }
}

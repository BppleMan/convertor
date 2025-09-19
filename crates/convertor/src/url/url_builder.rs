use crate::common::encrypt::encrypt;
use crate::config::client_config::ProxyClient;
use crate::config::provider_config::Provider;
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_header::SurgeHeader;
use crate::error::UrlBuilderError;
use crate::url::convertor_url::{ConvertorUrl, ConvertorUrlType};
use crate::url::query::ConvertorQuery;
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

    pub fn from_convertor_query(
        query: ConvertorQuery,
        secret: impl AsRef<str>,
        client: ProxyClient,
        provider: Provider,
    ) -> Result<Self, UrlBuilderError> {
        let ConvertorQuery {
            server,
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

    pub fn build_raw_url(&self) -> ConvertorUrl {
        let mut url = self.sub_url.clone();
        url.query_pairs_mut().append_pair("flag", self.client.as_ref());
        ConvertorUrl::raw(url)
    }

    pub fn build_raw_profile_url(&self) -> Result<ConvertorUrl, UrlBuilderError> {
        let query = self.as_profile_query().encode_to_profile_query()?;
        let url = ConvertorUrl::raw_profile(
            self.server.clone(),
            format!(
                "/raw-profile/{}/{}",
                self.client.as_ref().to_ascii_lowercase(),
                self.provider.as_ref().to_ascii_lowercase()
            ),
            query,
        );
        Ok(url)
    }

    pub fn build_profile_url(&self) -> Result<ConvertorUrl, UrlBuilderError> {
        let query = self.as_profile_query().encode_to_profile_query()?;
        let url = ConvertorUrl::profile(
            self.server.clone(),
            format!(
                "/profile/{}/{}",
                self.client.as_ref().to_ascii_lowercase(),
                self.provider.as_ref().to_ascii_lowercase()
            ),
            query,
        );
        Ok(url)
    }

    pub fn build_rule_provider_url(&self, policy: &Policy) -> Result<ConvertorUrl, UrlBuilderError> {
        let query = self.as_rule_provider_query(policy).encode_to_rule_provider_query()?;
        let url = ConvertorUrl::rule_provider(
            policy.clone(),
            self.server.clone(),
            format!(
                "/rule-provider/{}/{}",
                self.client.as_ref().to_ascii_lowercase(),
                self.provider.as_ref().to_ascii_lowercase()
            ),
            query,
        );
        Ok(url)
    }

    pub fn build_sub_logs_url(&self) -> Result<ConvertorUrl, UrlBuilderError> {
        let query = self.as_sub_logs_query().encode_to_sub_logs_query()?;
        let url = ConvertorUrl::sub_logs(self.server.clone(), "/api/sub-logs", query);
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
    fn host_port(&self) -> Option<String>;
}

impl HostPort for Url {
    fn host_port(&self) -> Option<String> {
        match (self.host_str(), self.port()) {
            (Some(host), Some(port)) => Some(format!("{host}:{port}")),
            (Some(host), None) => Some(host.to_string()),
            _ => None,
        }
    }
}

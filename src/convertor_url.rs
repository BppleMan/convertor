use crate::config::surge_config::RuleSetType;
use crate::encrypt::{decrypt, encrypt};
use crate::op::get_convertor_secret;
use axum::body::Body;
use axum::http::{header, Request};
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Report;
use reqwest::{IntoUrl, Url};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConvertorUrl {
    pub server: String,
    pub service_url: Url,
}

impl ConvertorUrl {
    /// 传入服务器地址和服务的 URL，生成 ConvertorUrl 实例
    /// 服务的 URL 指的是机场的订阅地址，通常包含 token
    pub fn new(
        server_addr: impl AsRef<str>,
        service_url: Url,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            server: server_addr.as_ref().to_string(),
            service_url,
        })
    }

    pub fn decode_from_convertor_url(
        convertor_url: impl IntoUrl,
    ) -> color_eyre::Result<Self> {
        let convertor_url = convertor_url.into_url()?;
        let server = convertor_url.origin().ascii_serialization().to_string();
        let convertor_secret = get_convertor_secret()?;
        let decoded_service_url = convertor_url
            .query_pairs()
            .find(|(k, _)| k == "raw_url")
            .ok_or_else(|| eyre!("raw_url not found"))?
            .1;
        let decrypted_service_url =
            decrypt(convertor_secret.as_bytes(), &decoded_service_url)?;
        let service_url = Url::parse(&decrypted_service_url)
            .wrap_err("Invalid service URL")?;
        Ok(Self {
            server,
            service_url,
        })
    }

    pub fn build_convertor_url(
        &self,
        flag: impl AsRef<str>,
    ) -> color_eyre::Result<Url> {
        let convertor_secret = get_convertor_secret()?;
        let encrypted_service_url =
            encrypt(convertor_secret.as_bytes(), self.service_url.as_str())?;
        let mut url = Url::parse(&self.server)?.join(flag.as_ref())?;
        url.query_pairs_mut()
            .append_pair("raw_url", &encrypted_service_url);
        Ok(url)
    }

    /// 构建一个规则集的 URL，用于获取机场的规则集
    pub fn build_rule_set_url(
        &self,
        rule_set_type: &RuleSetType,
    ) -> color_eyre::Result<Url> {
        let convertor_secret = get_convertor_secret()?;
        let encrypted_service_url =
            encrypt(&convertor_secret.as_bytes(), self.service_url.as_str())?;
        let mut url = Url::parse(&self.server)?.join("surge/rule_set")?;
        url.query_pairs_mut()
            .append_pair("raw_url", &encrypted_service_url);
        if matches!(rule_set_type, RuleSetType::BosLifeSubscription) {
            url.query_pairs_mut().append_pair("boslife", "true");
        } else {
            url.query_pairs_mut()
                .append_pair("policies", rule_set_type.policy());
        }
        Ok(url)
    }

    /// 构建一个服务的 URL，用于获取机场订阅
    pub fn build_subscription_url(
        &self,
        flag: impl AsRef<str>,
    ) -> color_eyre::Result<Url> {
        let mut url = self.service_url.clone();
        url.query_pairs_mut().append_pair("flag", flag.as_ref());
        Ok(url)
    }
}

impl TryFrom<&Request<Body>> for ConvertorUrl {
    type Error = Report;

    fn try_from(value: &Request<Body>) -> Result<Self, Self::Error> {
        let host = value
            .headers()
            .get(header::HOST)
            .ok_or_else(|| eyre!("Missing Host header"))?
            .to_str()?;
        let full_url = format!("http://{}{}", host, value.uri());
        ConvertorUrl::decode_from_convertor_url(full_url)
    }
}

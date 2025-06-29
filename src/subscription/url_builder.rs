use crate::encrypt::{decrypt, encrypt};
use crate::profile::rule_set_policy::RuleSetPolicy;
use axum::body::Body;
use axum::http::{header, Request};
use color_eyre::eyre::{eyre, WrapErr};
use reqwest::{IntoUrl, Url};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UrlBuilder {
    pub server: Url,
    pub service_url: Url,
    pub encrypted_service_url: String,
}

impl UrlBuilder {
    /// 传入服务器地址和服务的 URL，生成 ConvertorUrl 实例
    /// 服务的 URL 指的是机场的订阅地址，通常包含 token
    pub fn new(server: Url, secret: impl AsRef<str>, service_url: Url) -> color_eyre::Result<Self> {
        let encrypted_service_url = encrypt(secret.as_ref().as_bytes(), service_url.as_str())?;
        Ok(Self {
            server,
            service_url,
            encrypted_service_url,
        })
    }

    pub fn decode_from_convertor_url(
        convertor_url: impl IntoUrl,
        convertor_secret: impl AsRef<str>,
    ) -> color_eyre::Result<Self> {
        let convertor_url = convertor_url.into_url()?;
        let server = Url::parse(&convertor_url.origin().ascii_serialization())?;
        let encrypted_service_url = convertor_url
            .query_pairs()
            .find(|(k, _)| k == "raw_url")
            .ok_or_else(|| eyre!("raw_url not found"))?
            .1
            .to_string();
        let decrypted_service_url = decrypt(convertor_secret.as_ref().as_bytes(), &encrypted_service_url)?;
        let service_url = Url::parse(&decrypted_service_url).wrap_err("Invalid service URL")?;
        Ok(Self {
            server,
            service_url,
            encrypted_service_url,
        })
    }

    pub fn decode_from_request(request: &Request<Body>, convertor_secret: impl AsRef<str>) -> color_eyre::Result<Self> {
        let host = request
            .headers()
            .get(header::HOST)
            .ok_or_else(|| eyre!("Missing Host header"))?
            .to_str()?;
        let full_url = format!("http://{}{}", host, request.uri());
        UrlBuilder::decode_from_convertor_url(full_url, convertor_secret)
    }

    pub fn build_convertor_url(&self, flag: impl AsRef<str>) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push(flag.as_ref());
        }
        url.query_pairs_mut()
            .append_pair("raw_url", &self.encrypted_service_url);
        Ok(url)
    }

    pub fn build_proxy_provider_url(&self, flag: impl AsRef<str>) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push(flag.as_ref()).push("proxy-provider");
        }
        url.query_pairs_mut()
            .append_pair("raw_url", &self.encrypted_service_url);
        url.query_pairs_mut().append_pair("flag", flag.as_ref());
        Ok(url)
    }

    /// 构建一个规则集的 URL，用于获取机场的规则集
    pub fn build_rule_set_url(&self, flag: impl AsRef<str>, rule_set_type: &RuleSetPolicy) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push(flag.as_ref()).push("rule-set");
        }
        url.query_pairs_mut()
            .append_pair("raw_url", &self.encrypted_service_url);
        if matches!(rule_set_type, RuleSetPolicy::BosLifeSubscription) {
            url.query_pairs_mut().append_pair("boslife", "true");
        } else {
            url.query_pairs_mut().append_pair("policies", rule_set_type.policy());
        }
        Ok(url)
    }

    /// 构建一个服务的 URL，用于获取机场订阅
    pub fn build_subscription_url(&self, flag: impl AsRef<str>) -> color_eyre::Result<Url> {
        let mut url = self.service_url.clone();
        url.query_pairs_mut().append_pair("flag", flag.as_ref());
        Ok(url)
    }
}

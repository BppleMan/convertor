use crate::client::Client;
use crate::encrypt::{decrypt, encrypt};
use crate::profile::core::policy::{Policy, QueryPolicy};
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

    pub fn sub_host(&self) -> color_eyre::Result<String> {
        self.service_url
            .host_str()
            .and_then(|host| self.service_url.port().map(|port| format!("{}:{}", host, port)))
            .ok_or_else(|| eyre!("服务 URL 无效"))
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

    pub fn build_convertor_url(&self, client: Client) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push("profile");
        }
        url.query_pairs_mut()
            .append_pair("raw_url", &self.encrypted_service_url)
            .append_pair("client", client.as_str());
        Ok(url)
    }

    /// 构建一个规则集的 URL，用于获取机场的规则集
    pub fn build_rule_set_url(&self, client: Client, policy: &Policy) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push("rule-set");
        }

        let query_policy = QueryPolicy::from(policy.clone());
        let policy = serde_urlencoded::to_string(&query_policy)?;
        url.query_pairs_mut()
            .append_pair("client", client.as_str())
            .append_pair("raw_url", &self.encrypted_service_url);
        let policy_pair = url::form_urlencoded::parse(policy.as_bytes());
        policy_pair.for_each(|(k, v)| {
            url.query_pairs_mut().append_pair(&k, &v);
        });
        Ok(url)
    }

    /// 构建一个服务的 URL，用于获取机场订阅
    pub fn build_subscription_url(&self, client: Client) -> color_eyre::Result<Url> {
        let mut url = self.service_url.clone();
        // BosLife 的字段是 `flag` 不可改为client
        url.query_pairs_mut().append_pair("flag", client.as_str());
        Ok(url)
    }
}

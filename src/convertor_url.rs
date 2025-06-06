use crate::boslife::RuleSetType;
use crate::encrypt::{decrypt, encrypt};
use crate::op;
use crate::op::get_convertor_secret;
use color_eyre::eyre::eyre;
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorUrl {
    pub server: String,
    pub service_url: String,
    pub service_token: String,
    pub encrypted_service_token: String,
}

impl ConvertorUrl {
    /// 传入服务器地址和服务的 URL，生成 ConvertorUrl 实例
    /// 服务的 URL 指的是机场的订阅地址，通常包含 token
    pub fn new(
        server_addr: impl AsRef<str>,
        service_url: &Url,
    ) -> color_eyre::Result<Self> {
        let base_url = format!(
            "{}{}",
            service_url.origin().ascii_serialization(),
            service_url.path()
        );
        let token = service_url
            .query_pairs()
            .find(|(k, _)| k == "token")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| eyre!("Token not found"))?;
        let secret = op::get_convertor_secret()?;
        let encrypted_token = encrypt(secret.as_ref(), &token)?;
        Ok(Self {
            server: server_addr.as_ref().to_string(),
            service_url: base_url,
            service_token: token,
            encrypted_service_token: encrypted_token,
        })
    }

    pub fn decode_from_convertor_url(
        convertor_url: impl IntoUrl,
    ) -> color_eyre::Result<Self> {
        let convertor_url = convertor_url.into_url()?;
        let server = convertor_url.origin().ascii_serialization().to_string();
        let airport_url = convertor_url
            .query_pairs()
            .find(|(k, _)| k == "base_url")
            .map(|(_, v)| urlencoding::decode(&v).unwrap().to_string())
            .ok_or_else(|| eyre!("base_url not found"))?;
        let encrypted_token = convertor_url
            .query_pairs()
            .find(|(k, _)| k == "token")
            .map(|(_, v)| urlencoding::decode(&v).unwrap().to_string())
            .ok_or_else(|| eyre!("token not found"))?;
        let secret = get_convertor_secret()?;
        let airport_token = decrypt(secret.as_ref(), &encrypted_token)?;
        Ok(Self {
            server,
            service_url: airport_url,
            service_token: airport_token,
            encrypted_service_token: encrypted_token,
        })
    }

    pub fn encode_to_convertor_url(&self) -> color_eyre::Result<Url> {
        let mut url = Url::parse(&self.server)?.join("surge")?;
        url.query_pairs_mut()
            .append_pair("base_url", &urlencoding::encode(&self.service_url))
            .append_pair(
                "token",
                &urlencoding::encode(&self.encrypted_service_token),
            );
        Ok(url)
    }

    pub fn create_rule_set_api(
        &self,
        rule_set_type: &RuleSetType,
    ) -> color_eyre::Result<Url> {
        let mut url = Url::parse(&self.server)?.join("surge/rule_set")?;
        url.query_pairs_mut()
            .append_pair("base_url", &self.service_url)
            .append_pair("token", &self.service_token);
        match rule_set_type {
            RuleSetType::BosLifeSubscription => {
                url.query_pairs_mut().append_pair("boslife", "true")
            }
            RuleSetType::BosLifePolicy => {
                url.query_pairs_mut().append_pair("policies", "BosLife")
            }
            RuleSetType::BosLifeNoResolvePolicy => url
                .query_pairs_mut()
                .append_pair("policies", "BosLife|no-resolve"),
            RuleSetType::BosLifeForceRemoteDnsPolicy => url
                .query_pairs_mut()
                .append_pair("policies", "BosLife|force-remote-dns"),
            RuleSetType::DirectPolicy => {
                url.query_pairs_mut().append_pair("policies", "DIRECT")
            }
            RuleSetType::DirectNoResolvePolicy => url
                .query_pairs_mut()
                .append_pair("policies", "DIRECT|no-resolve"),
            RuleSetType::DirectForceRemoteDnsPolicy => url
                .query_pairs_mut()
                .append_pair("policies", "DIRECT|force-remote-dns"),
        };
        Ok(url)
    }

    /// 构建一个服务的 URL，用于获取机场订阅
    pub fn build_service_url(
        &self,
        flag: impl AsRef<str>,
    ) -> color_eyre::Result<Url> {
        let mut url = Url::parse(&self.service_url)?;
        url.query_pairs_mut()
            .append_pair("token", &self.service_token);
        url.query_pairs_mut().append_pair("flag", flag.as_ref());
        Ok(url)
    }
}

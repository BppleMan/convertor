use crate::boslife::RuleSetType;
use crate::encrypt::{decrypt, encrypt};
use color_eyre::eyre::{eyre, WrapErr};
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertorUrl {
    pub server: String,
    pub airport_url: String,
    pub airport_token: String,
    pub encrypted_token: String,
}

impl ConvertorUrl {
    pub(crate) fn new(
        server_addr: impl AsRef<str>,
        subscription_url: &Url,
    ) -> color_eyre::Result<Self> {
        let base_url = format!(
            "{}{}",
            subscription_url.origin().ascii_serialization(),
            subscription_url.path()
        );
        let token = subscription_url
            .query_pairs()
            .find(|(k, _)| k == "token")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| eyre!("token not found"))?;
        let secret = std::env::var("CONVERTOR_SECRET")?;
        let encrypted_token = encrypt(secret.as_ref(), &token)?;
        Ok(Self {
            server: server_addr.as_ref().to_string(),
            airport_url: base_url,
            airport_token: token,
            encrypted_token,
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
        let secret = std::env::var("CONVERTOR_SECRET")
            .wrap_err("没有找到环境变量 $CONVERTOR_SECRET")
            .;
        let airport_token = decrypt(secret.as_ref(), &encrypted_token)?;
        Ok(Self {
            server,
            airport_url,
            airport_token,
            encrypted_token,
        })
    }

    pub fn encode_to_convertor_url(&self) -> color_eyre::Result<Url> {
        let mut url = Url::parse(&self.server)?.join("surge")?;
        url.query_pairs_mut()
            .append_pair("base_url", &urlencoding::encode(&self.airport_url))
            .append_pair("token", &urlencoding::encode(&self.encrypted_token));
        Ok(url)
    }

    pub fn create_rule_set_api(
        &self,
        rule_set_type: &RuleSetType,
    ) -> color_eyre::Result<Url> {
        let mut url = Url::parse(&self.server)?.join("surge/rule_set")?;
        url.query_pairs_mut()
            .append_pair("base_url", &self.airport_url)
            .append_pair("token", &self.airport_token);
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
    
    pub fn build_airport_url(
        &self,
        flag: impl AsRef<str>,
    ) -> color_eyre::Result<Url> {
        let mut url = Url::parse(&self.airport_url)?;
        url.query_pairs_mut().append_pair("token", &secret);
        url.query_pairs_mut().append_pair("flag", flag.as_ref());
        Ok(url)
    }
}

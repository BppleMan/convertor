use crate::client::Client;
use crate::encrypt::{decrypt, encrypt};
use crate::profile::core::policy::Policy;
use crate::server::query::{ProfileQuery, QueryPolicy};
use color_eyre::eyre::{eyre, ContextCompat, WrapErr};
use percent_encoding::{percent_decode_str, utf8_percent_encode, CONTROLS, NON_ALPHANUMERIC};
use reqwest::{IntoUrl, Url};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UrlBuilder {
    pub server: Url,
    pub raw_sub_url: Url,
    pub encrypted_raw_sub_url: String,
}

impl UrlBuilder {
    /// 传入服务器地址和服务的 URL，生成 ConvertorUrl 实例
    /// 服务的 URL 指的是机场的订阅地址，通常包含 token
    pub fn new(server: Url, secret: impl AsRef<str>, raw_sub_url: impl IntoUrl) -> color_eyre::Result<Self> {
        let secret = secret.as_ref().to_string();
        let raw_sub_url = raw_sub_url.into_url()?;
        let encrypted_raw_sub_url = encrypt(secret.as_bytes(), raw_sub_url.as_str())?;
        Ok(Self {
            server,
            raw_sub_url,
            encrypted_raw_sub_url,
        })
    }

    pub fn decode_from_convertor_url(
        convertor_url: impl IntoUrl,
        convertor_secret: impl AsRef<str>,
    ) -> color_eyre::Result<Self> {
        let convertor_url = convertor_url.into_url()?;
        let profile_query = convertor_url
            .query()
            .map(serde_qs::from_str::<ProfileQuery>)
            .wrap_err("无法解析 Convertor URL 查询参数")??;
        Self::decode_from_query(&profile_query, convertor_secret)
    }

    pub fn decode_from_query(
        profile_query: &ProfileQuery,
        convertor_secret: impl AsRef<str>,
    ) -> color_eyre::Result<Self> {
        let server_str = percent_decode_str(profile_query.original_host.as_str())
            .decode_utf8()?
            .to_string();
        let server = Url::parse(&server_str).wrap_err("server 无法解析为 url")?;
        let encrypted_raw_sub_url = percent_decode_str(profile_query.raw_sub_url.as_str())
            .decode_utf8()?
            .to_string();
        let decrypted_raw_sub_url = decrypt(convertor_secret.as_ref().as_bytes(), &encrypted_raw_sub_url)?;
        let raw_sub_url = Url::parse(&decrypted_raw_sub_url).wrap_err("raw_sub_url 无法解析为 url")?;
        Ok(Self {
            server,
            raw_sub_url,
            encrypted_raw_sub_url,
        })
    }

    pub fn encode_to_profile_query(
        &self,
        client: Client,
        policy: Option<impl Into<QueryPolicy>>,
    ) -> color_eyre::Result<ProfileQuery> {
        Ok(ProfileQuery {
            client,
            original_host: utf8_percent_encode(self.server.as_str(), CONTROLS).to_string(),
            raw_sub_url: utf8_percent_encode(self.encrypted_raw_sub_url.as_str(), CONTROLS).to_string(),
            policy: policy.map(|p| p.into()),
        })
    }

    pub fn sub_host(&self) -> color_eyre::Result<String> {
        self.raw_sub_url
            .host_str()
            .and_then(|host| self.raw_sub_url.port().map(|port| format!("{}:{}", host, port)))
            .ok_or_else(|| eyre!("服务 URL 无效"))
    }

    pub fn build_convertor_url(&self, client: Client) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push("profile");
        }
        let profile_query = self.encode_to_profile_query(client, Option::<QueryPolicy>::None)?;
        let query_string = serde_qs::to_string(&profile_query)?;
        let encoded_query = utf8_percent_encode(&query_string, NON_ALPHANUMERIC).to_string();
        url.set_query(Some(&encoded_query));
        Ok(url)
    }

    /// 构建一个规则集的 URL，用于获取机场的规则集
    pub fn build_rule_provider_url(&self, client: Client, policy: &Policy) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push("rule-provider");
        }

        let profile_query = self.encode_to_profile_query(client, Some(policy.clone()))?;
        let query_string = serde_qs::to_string(&profile_query)?;
        let encoded_query = utf8_percent_encode(&query_string, NON_ALPHANUMERIC).to_string();
        url.set_query(Some(&encoded_query));
        Ok(url)
    }

    /// 构建一个服务的 URL，用于获取机场订阅
    pub fn build_subscription_url(&self, client: Client) -> color_eyre::Result<Url> {
        let mut url = self.raw_sub_url.clone();
        // BosLife 的字段是 `flag` 不可改为client
        url.query_pairs_mut().append_pair("flag", client.as_str());
        Ok(url)
    }
}

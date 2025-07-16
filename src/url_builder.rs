use crate::client::Client;
use crate::core::profile::policy::Policy;
use crate::encrypt::{decrypt, encrypt};
use crate::router::query::{ProfileQuery, QueryPolicy, SubLogQuery};
use color_eyre::eyre::{WrapErr, eyre};
use percent_encoding::{CONTROLS, PercentDecode, percent_decode_str, utf8_percent_encode};
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
            .map(percent_decode_str)
            .map(PercentDecode::decode_utf8)
            .ok_or_else(|| eyre!("Convertor url 必须包含查询参数"))?
            .map(ProfileQuery::decode_from_query_string)
            .wrap_err_with(|| eyre!("Convertor url 查询参数无法解码"))?
            .wrap_err_with(|| eyre!("Convertor url 查询参数无法反序列化"))?;
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
        let decrypted_raw_sub_url = decrypt(convertor_secret.as_ref().as_bytes(), &encrypted_raw_sub_url)
            .wrap_err_with(|| "无法解密原始订阅链接")?;
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
            interval: 86400,
            strict: true,
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
        let query_string = profile_query.encode_to_query_string();
        url.set_query(Some(&query_string));
        Ok(url)
    }

    /// 用于获取机场的规则集的 URL
    pub fn build_rule_provider_url(&self, client: Client, policy: &Policy) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push("rule-provider");
        }

        let profile_query = self.encode_to_profile_query(client, Some(policy.clone()))?;
        let query_string = profile_query.encode_to_query_string();
        let encoded_query = utf8_percent_encode(&query_string, CONTROLS).to_string();
        url.set_query(Some(&encoded_query));
        Ok(url)
    }

    /// 用于获取机场订阅日志的 URL
    pub fn build_sub_logs_url(&self, secret: impl AsRef<str>) -> color_eyre::Result<Url> {
        let mut url = self.server.clone();
        {
            let mut path = url.path_segments_mut().map_err(|()| eyre!("无法获取路径段"))?;
            path.push("sub-logs");
        }
        let sub_log_query = SubLogQuery::new(secret, Some(1), Some(10));
        let query_string = sub_log_query.encode_to_query_string()?;
        url.set_query(Some(&query_string));
        Ok(url)
    }

    /// 用于获取机场订阅的 URL
    pub fn build_subscription_url(&self, client: Client) -> color_eyre::Result<Url> {
        let mut url = self.raw_sub_url.clone();
        // BosLife 的字段是 `flag` 不可改为client
        url.query_pairs_mut().append_pair("flag", client.as_str());
        Ok(url)
    }

    pub fn encode_encrypted_raw_sub_url(&self) -> String {
        utf8_percent_encode(&self.encrypted_raw_sub_url, CONTROLS).to_string()
    }
}

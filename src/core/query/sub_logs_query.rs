use crate::common::config::sub_provider::SubProvider;
use crate::common::encrypt::decrypt;
use color_eyre::eyre::{OptionExt, WrapErr};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubLogsQuery {
    pub provider: SubProvider,
    pub secret: String,
    pub enc_secret: String,
    pub page: usize,
    pub page_size: usize,
}

impl SubLogsQuery {
    pub fn new(
        provider: SubProvider,
        secret: impl AsRef<str>,
        enc_secret: String,
        page: usize,
        page_size: usize,
    ) -> Self {
        let secret = secret.as_ref().to_string();
        SubLogsQuery {
            provider,
            secret,
            enc_secret,
            page,
            page_size,
        }
    }

    pub fn encode_to_query_string(&self) -> String {
        let encoded_secret = utf8_percent_encode(&self.enc_secret, percent_encoding::CONTROLS).to_string();
        let query_pairs = [
            format!("provider={}", self.provider),
            format!("secret={}", encoded_secret),
            format!("page={}", self.page),
            format!("page_size={}", self.page_size),
        ];
        query_pairs.join("&")
    }

    pub fn decode_from_query_string(
        query_string: impl AsRef<str>,
        secret: impl AsRef<str>,
    ) -> color_eyre::Result<SubLogsQuery> {
        let query_pairs = query_string
            .as_ref()
            .split('&')
            .filter_map(|p| p.split_once('='))
            .collect::<HashMap<_, _>>();

        let provider = query_pairs
            .get("provider")
            .ok_or_eyre("缺少 provider 参数")?
            .parse::<SubProvider>()
            .wrap_err("无法解析 provider 参数")?;
        let encoded_secret = query_pairs.get("secret").ok_or_eyre("缺少 secret 参数")?;
        let enc_secret = percent_decode_str(encoded_secret)
            .decode_utf8()
            .wrap_err("无法进行 url decoding")?
            .to_string();
        let secret = decrypt(secret.as_ref().as_bytes(), enc_secret.as_ref())?;

        let page = query_pairs
            .get("page")
            .map(|s| s.parse::<usize>())
            .transpose()
            .wrap_err("无法解析 page 为 num")?
            .ok_or_eyre("缺少 page 参数")?;
        let page_size = query_pairs
            .get("page_size")
            .map(|s| s.parse::<usize>())
            .transpose()
            .wrap_err("无法解析 page_size 为 num")?
            .ok_or_eyre("缺少 page_size 参数")?;

        Ok(SubLogsQuery {
            provider,
            secret,
            enc_secret,
            page,
            page_size,
        })
    }
}

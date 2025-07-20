use crate::common::encrypt::{decrypt, encrypt};
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SubLogQuery {
    pub secret: String,
    pub page_current: usize,
    pub page_size: usize,
}

impl SubLogQuery {
    pub fn new(secret: impl AsRef<str>, page_current: usize, page_size: usize) -> Self {
        let secret = secret.as_ref().to_string();
        SubLogQuery {
            secret,
            page_current,
            page_size,
        }
    }

    pub fn encode_to_query_string(&self) -> Result<String> {
        let encrypted_secret = encrypt(self.secret.as_bytes(), self.secret.as_str())?;
        let encoded_secret = utf8_percent_encode(&encrypted_secret, percent_encoding::CONTROLS).to_string();
        let query_pairs = [
            format!("secret={}", encoded_secret),
            format!("page_current={}", self.page_current),
            format!("page_size={}", self.page_size),
        ];
        Ok(query_pairs.join("&"))
    }

    pub fn decode_from_query_string(query_string: impl AsRef<str>, secret: impl AsRef<str>) -> Result<SubLogQuery> {
        let query_pairs = query_string
            .as_ref()
            .split('&')
            .filter_map(|p| p.split_once('='))
            .collect::<HashMap<_, _>>();

        let encoded_secret = query_pairs.get("secret").ok_or_eyre("缺少 secret 参数")?;
        let encrypted_secret = percent_decode_str(encoded_secret)
            .decode_utf8()
            .wrap_err("无法进行 url decoding")?;
        let secret = decrypt(secret.as_ref().as_bytes(), encrypted_secret.as_ref())?;

        let page_current = query_pairs
            .get("page_current")
            .map(|s| s.parse::<usize>())
            .transpose()
            .wrap_err("无法解析 page_current 为 num")?
            .ok_or_eyre("缺少 page_current 参数")?;
        let page_size = query_pairs
            .get("page_size")
            .map(|s| s.parse::<usize>())
            .transpose()
            .wrap_err("无法解析 page_current 为 num")?
            .ok_or_eyre("缺少 page_size 参数")?;

        Ok(SubLogQuery {
            secret,
            page_current,
            page_size,
        })
    }
}

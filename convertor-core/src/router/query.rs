use crate::encrypt::{decrypt, encrypt};
use color_eyre::Result;
use color_eyre::eyre::{OptionExt, WrapErr};
use percent_encoding::{percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubLogQuery {
    pub secret: String,
    pub page_current: Option<usize>,
    pub page_size: Option<usize>,
}

impl SubLogQuery {
    pub fn new(secret: impl AsRef<str>, page_current: Option<usize>, page_size: Option<usize>) -> Self {
        SubLogQuery {
            secret: secret.as_ref().to_string(),
            page_current,
            page_size,
        }
    }

    pub fn encode_to_query_string(&self) -> Result<String> {
        let mut query_pairs = vec![];
        let encrypted_secret = encrypt(self.secret.as_bytes(), self.secret.as_str())?;
        query_pairs.push(format!(
            "secret={}",
            utf8_percent_encode(&encrypted_secret, percent_encoding::CONTROLS)
        ));
        if let Some(page_current) = self.page_current {
            query_pairs.push(format!("page_current={page_current}"));
        }
        if let Some(page_size) = self.page_size {
            query_pairs.push(format!("page_size={page_size}"));
        }
        query_pairs.sort();
        Ok(query_pairs.join("&"))
    }

    pub fn decode_from_query_string(query_string: impl AsRef<str>, secret: impl AsRef<str>) -> Result<SubLogQuery> {
        let query_pairs = query_string
            .as_ref()
            .split('&')
            .filter_map(|p| p.split_once('='))
            .collect::<HashMap<_, _>>();
        let secret = query_pairs
            .get("secret")
            .map(|s| percent_decode_str(s).decode_utf8())
            .transpose()
            .wrap_err("无法进行 url decoding")?
            .map(|s| decrypt(secret.as_ref().as_bytes(), s.as_ref()))
            .transpose()
            .wrap_err("未认证的 secret")?
            .ok_or_eyre("缺少 secret 参数")?
            .to_string();
        let page_current = query_pairs.get("page_current").and_then(|s| s.parse::<usize>().ok());
        let page_size = query_pairs.get("page_size").and_then(|s| s.parse::<usize>().ok());

        Ok(SubLogQuery {
            secret,
            page_current,
            page_size,
        })
    }
}

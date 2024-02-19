use crate::error::AppError;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::Url;
use std::str::FromStr;

pub mod clash;
pub mod surge;

pub async fn root() -> Result<(), AppError> {
    Err(eyre!("Hello, World!"))?;
    Ok(())
}

pub async fn get_raw_profile(
    url: impl AsRef<str>,
    flag: impl AsRef<str>,
) -> Result<String> {
    let mut url = Url::from_str(url.as_ref())?;
    url.query_pairs_mut().append_pair("flag", flag.as_ref());
    reqwest::Client::new()
        .get(url)
        .header(
            "User-Agent",
            format!("convertor/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await?
        .text()
        .await
        .map_err(Into::into)
}

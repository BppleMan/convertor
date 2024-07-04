use indexmap::IndexMap;
use reqwest::Url;
use std::str::FromStr;
use trie_rs::TrieBuilder;

pub mod surge_profile;
pub mod clash_profile;

fn group_by(sources: &[impl AsRef<str>]) -> IndexMap<String, Vec<String>> {
    let trie = sources
        .iter()
        .fold(TrieBuilder::new(), |mut acc, proxy| {
            acc.push(proxy.as_ref());
            acc
        })
        .build();
    sources.iter().fold(
        IndexMap::<String, Vec<String>>::new(),
        |mut acc, source| {
            let source = source.as_ref();
            let unselect = acc
                .iter()
                .all(|(_key, value)| !value.contains(&source.to_string()));
            if unselect {
                if let Some((prefix, _)) = source.rsplit_once('-') {
                    // let results_in_u8s = trie.predictive_search(prefix).collect::<Vec<Vec<u8>>>();
                    let results_in_str =
                        trie.predictive_search(prefix).collect::<Vec<String>>();
                    acc.insert(prefix.to_string(), results_in_str);
                } else {
                    acc.insert(source.to_string(), vec![source.to_string()]);
                }
            }
            acc
        },
    )
}

pub async fn get_raw_profile(
    url: impl AsRef<str>,
    flag: impl AsRef<str>,
) -> color_eyre::Result<String> {
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

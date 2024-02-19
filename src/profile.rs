use indexmap::IndexMap;
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
                    let results_in_u8s = trie.predictive_search(prefix);
                    let results_in_str = results_in_u8s
                        .iter()
                        .map(|u8s| String::from_utf8(u8s.clone()).unwrap())
                        .collect::<Vec<_>>();
                    acc.insert(prefix.to_string(), results_in_str);
                } else {
                    acc.insert(source.to_string(), vec![source.to_string()]);
                }
            }
            acc
        },
    )
}

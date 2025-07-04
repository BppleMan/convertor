use crate::region::Region;
use indexmap::IndexMap;
use regex::Regex;

pub mod surge_profile;
pub mod clash_profile;
pub mod rule_set_policy;

pub fn group_by_region<S: AsRef<str>>(sources: &[S]) -> IndexMap<String, Vec<String>> {
    let match_number = Regex::new(r"\W*\d+\s*$").unwrap();
    sources
        .iter()
        .fold(IndexMap::<String, Vec<String>>::new(), |mut acc, source| {
            let source = source.as_ref();
            let region_part = match_number.replace(source, "").to_string();
            acc.entry(region_part).or_default().push(source.to_string());
            acc
        })
}

pub fn split_and_merge_groups(
    groups: IndexMap<String, Vec<String>>,
) -> (IndexMap<&'static Region, Vec<String>>, Vec<String>) {
    let mut useful_groups: IndexMap<&'static Region, Vec<String>> = IndexMap::new();
    let mut extra_groups = vec![];

    for group_name in groups.keys() {
        if let Some(region) = Region::detect(group_name) {
            useful_groups
                .entry(region)
                .or_default()
                .extend(groups[group_name].clone());
        } else {
            extra_groups.extend(groups[group_name].clone());
        }
    }

    (useful_groups, extra_groups)
}

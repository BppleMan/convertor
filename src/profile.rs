pub mod surge_profile;
pub mod clash_profile;
pub mod rule_set_policy;
pub(self) mod proxy;
pub(self) mod proxy_group;
pub(self) mod rule;
pub(self) mod rule_provider;
// pub fn split_and_merge_groups(
//     groups: IndexMap<String, Vec<String>>,
// ) -> (IndexMap<&'static Region, Vec<String>>, Vec<String>) {
//     let mut useful_groups: IndexMap<&'static Region, Vec<String>> = IndexMap::new();
//     let mut extra_groups = vec![];
//
//     for group_name in groups.keys() {
//         if let Some(region) = Region::detect(group_name) {
//             useful_groups
//                 .entry(region)
//                 .or_default()
//                 .extend(groups[group_name].clone());
//         } else {
//             extra_groups.extend(groups[group_name].clone());
//         }
//     }
//
//     (useful_groups, extra_groups)
// }

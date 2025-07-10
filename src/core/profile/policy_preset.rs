// use crate::profile::policy::Policy;
// use color_eyre::eyre::eyre;
// use color_eyre::Report;
// use std::str::FromStr;
//
// #[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
// pub enum PolicyPreset {
//     Subscription,
//     Direct,
//     DirectNoResolve,
//     DirectForceRemoteDns,
//     Other(Policy),
// }
//
// impl PolicyPreset {
//     pub const fn all() -> &'static [PolicyPreset] {
//         &[
//             PolicyPreset::Subscription,
//             PolicyPreset::Direct,
//             PolicyPreset::DirectNoResolve,
//             PolicyPreset::DirectForceRemoteDns,
//         ]
//     }
//
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             PolicyPreset::Subscription => "Subscription",
//             PolicyPreset::Direct => "Direct",
//             PolicyPreset::DirectNoResolve => "DirectNoResolve",
//             PolicyPreset::DirectForceRemoteDns => "DirectForceRemoteDns",
//             PolicyPreset::Other(_) => "Other",
//         }
//     }
//
//     pub fn provider_name(&self) -> &'static str {
//         self.as_str()
//     }
//
//     pub fn section_name(&self) -> &'static str {
//         match self {
//             PolicyPreset::Subscription => "[BosLife Subscription]",
//             PolicyPreset::Direct => "[Direct Policy]",
//             PolicyPreset::DirectNoResolve => "[Direct No Resolve Policy]",
//             PolicyPreset::DirectForceRemoteDns => "[Direct Force Remote Dns Policy]",
//         }
//     }
//
//     pub fn comment(&self) -> String {
//         format!(
//             r#"// Added for {} by convertor/{}"#,
//             self.section_name(),
//             env!("CARGO_PKG_VERSION")
//         )
//     }
// }
//
// impl FromStr for PolicyPreset {
//     type Err = Report;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "BosLife" => Ok(PolicyPreset::BosLife),
//             "BosLifeNoResolve" => Ok(PolicyPreset::BosLifeNoResolve),
//             "BosLifeForceRemoteDns" => Ok(PolicyPreset::BosLifeForceRemoteDns),
//             "Direct" => Ok(PolicyPreset::Direct),
//             "DirectNoResolve" => Ok(PolicyPreset::DirectNoResolve),
//             "DirectForceRemoteDns" => Ok(PolicyPreset::DirectForceRemoteDns),
//             _ => Err(eyre!("无法解析的规则集策略: {}", s)),
//         }
//     }
// }
//
// impl From<Policy> for PolicyPreset {
//     fn from(policy: Policy) -> Self {
//         match policy.name.as_str() {
//             "BosLife" => PolicyPreset::BosLife,
//             "BosLifeNoResolve" => PolicyPreset::BosLifeNoResolve,
//             "BosLifeForceRemoteDns" => PolicyPreset::BosLifeForceRemoteDns,
//             "Direct" => PolicyPreset::Direct,
//             "DirectNoResolve" => PolicyPreset::DirectNoResolve,
//             "DirectForceRemoteDns" => PolicyPreset::DirectForceRemoteDns,
//             _ => panic!("无法转换的规则集策略: {}", policy.name),
//         }
//     }
// }
//
// impl From<PolicyPreset> for Policy {
//     fn from(preset: PolicyPreset) -> Self {
//         match preset {
//             PolicyPreset::Subscription => Policy {
//                 name: "DIRECT".to_string(),
//                 option: None,
//             },
//             PolicyPreset::Direct => Policy {
//                 name: "DIRECT".to_string(),
//                 option: None,
//             },
//             PolicyPreset::DirectNoResolve => Policy {
//                 name: "DIRECT".to_string(),
//                 option: Some("no-resolve".to_string()),
//             },
//             PolicyPreset::DirectForceRemoteDns => Policy {
//                 name: "DIRECT".to_string(),
//                 option: Some("force-remote-dns".to_string()),
//             },
//             PolicyPreset::Other(policy) => policy,
//         }
//     }
// }

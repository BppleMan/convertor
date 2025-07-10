use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::profile::rule_provider::RuleProvider;
use crate::core::renderer::{INDENT, Renderer};
use crate::core::result::RenderResult;
use std::fmt::Write;
use tracing::instrument;

pub struct ClashRenderer;

impl Renderer for ClashRenderer {
    type PROFILE = ClashProfile;

    fn client() -> crate::client::Client {
        crate::client::Client::Clash
    }

    #[instrument(skip_all)]
    fn render_profile(profile: &Self::PROFILE) -> RenderResult<String> {
        let mut output = String::new();
        writeln!(output, "{}", Self::render_general(profile)?)?;

        let proxies = Self::render_proxies(&profile.proxies)?;
        writeln!(output, "proxies:")?;
        writeln!(output, "{}", proxies)?;

        let proxy_groups = Self::render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output, "proxy-groups:")?;
        writeln!(output, "{}", proxy_groups)?;

        let rule_providers = Self::render_rule_providers(&profile.rule_providers)?;
        writeln!(output, "rule-providers:")?;
        writeln!(output, "{}", rule_providers)?;

        let rules = Self::render_rules(&profile.rules)?;
        writeln!(output, "rules:")?;
        writeln!(output, "{}", rules)?;

        Ok(output)
    }

    #[instrument(skip_all)]
    fn render_general(profile: &Self::PROFILE) -> RenderResult<String> {
        let mut output = String::new();
        writeln!(output, "port: {}", profile.port)?;
        writeln!(output, "socks-port: {}", profile.socks_port)?;
        writeln!(output, "redir-port: {}", profile.redir_port)?;
        writeln!(output, "allow-lan: {}", profile.allow_lan)?;
        writeln!(output, "mode: {}", profile.mode)?;
        writeln!(output, "log-level: {}", profile.log_level)?;
        writeln!(output, r#"external-controller: "{}""#, profile.external_controller)?;
        writeln!(output, r#"secret: "{}""#, profile.secret)?;
        Ok(output)
    }

    fn render_proxy(proxy: &Proxy) -> RenderResult<String> {
        let mut output = String::new();
        write!(output, "{{ ")?;
        write!(output, r#"name: "{}""#, &proxy.name)?;
        write!(output, r#", type: "{}""#, &proxy.r#type)?;
        write!(output, r#", server: "{}""#, &proxy.server)?;
        write!(output, r#", port: {}"#, &proxy.port)?;
        write!(output, r#", password: "{}""#, &proxy.password)?;
        if let Some(udp) = &proxy.udp {
            write!(output, r#", udp: {}"#, udp)?;
        }
        if let Some(tfo) = &proxy.tfo {
            write!(output, r#", tfo: {}"#, tfo)?;
        }
        if let Some(cipher) = &proxy.cipher {
            write!(output, r#", cipher: {}"#, cipher)?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, r#", sni: "{}""#, sni)?;
        }
        if let Some(skip_cert_verify) = &proxy.skip_cert_verify {
            write!(output, r#", skip-cert-verify: {}"#, skip_cert_verify)?;
        }
        write!(output, " }}")?;
        Ok(output)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> RenderResult<String> {
        let mut output = String::new();
        write!(output, "{{ ")?;
        write!(output, r#"name: "{}""#, proxy_group.name)?;
        write!(output, r#", type: "{}""#, proxy_group.r#type.as_str())?;
        write!(output, r#", proxies: [ {} ]"#, proxy_group.proxies.join(", "))?;
        write!(output, " }}")?;
        Ok(output)
    }

    fn render_rule(rule: &Rule) -> RenderResult<String> {
        let mut output = String::new();
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(output, ",{}", value)?;
        }
        write!(output, ",{}", Self::render_policy(&rule.policy)?)?;
        Ok(output)
    }

    fn render_rule_for_provider(rule: &Rule) -> RenderResult<String> {
        Self::render_rule(rule)
    }

    fn render_provider_rule(rule: &ProviderRule) -> RenderResult<String> {
        Ok(format!("{},{}", rule.rule_type.as_str(), rule.value))
    }

    #[instrument(skip_all)]
    fn render_rule_providers(rule_providers: &[(String, RuleProvider)]) -> RenderResult<String> {
        let output = rule_providers
            .iter()
            .map(Self::render_rule_provider)
            .map(|line| line.map(|line| format!("{:indent$}{}", "", line, indent = INDENT)))
            .collect::<RenderResult<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    fn render_rule_provider(rule_provider: &(String, RuleProvider)) -> RenderResult<String> {
        let (name, rule_provider) = rule_provider;
        Ok(format!(
            r#"{}: {{ type: "{}", url: "{}", path: "{}", interval: {}, size-limit: {}, format: "{}", behavior: "{}" }}"#,
            name,
            rule_provider.r#type,
            rule_provider.url,
            rule_provider.path,
            rule_provider.interval,
            rule_provider.size_limit,
            rule_provider.format,
            rule_provider.behavior
        ))
    }

    fn render_provider_name_for_policy(policy: &Policy) -> RenderResult<String> {
        if policy == &Policy::subscription_policy() {
            return Ok("Subscription".to_string());
        }
        let output = format!(
            "{}_{}",
            policy.name,
            policy.option.as_ref().unwrap_or(&"policy".to_string())
        );
        Ok(output.replace("-", "_"))
    }
}

// impl ClashRenderer {
//     #[instrument(skip_all)]
//     pub fn render_profile(profile: &ClashProfile) -> Result<String> {
//         let ClashProfile {
//             proxies,
//             proxy_groups,
//             rules,
//             rule_providers,
//             ..
//         } = &profile;
//         let lines = [
//             Self::render_general(profile)?,
//             Self::render_proxies(proxies)?,
//             Self::render_proxy_groups(proxy_groups)?,
//             Self::render_rule_providers(rule_providers)?,
//             Self::render_rules(rules)?,
//         ]
//         .join("\n");
//         Ok(lines)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_general(profile: &ClashProfile) -> Result<String> {
//         let mut output = String::new();
//         writeln!(output, "port: {}", profile.port)?;
//         writeln!(output, "socks-port: {}", profile.socks_port)?;
//         writeln!(output, "redir-port: {}", profile.redir_port)?;
//         writeln!(output, "allow-lan: {}", profile.allow_lan)?;
//         writeln!(output, "mode: {}", profile.mode)?;
//         writeln!(output, "log-level: {}", profile.log_level)?;
//         writeln!(output, r#"external-controller: "{}""#, profile.external_controller)?;
//         writeln!(output, r#"secret: "{}""#, profile.secret)?;
//         writeln!(output)?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_proxies(proxies: &[Proxy]) -> Result<String> {
//         let mut output = String::new();
//         writeln!(output, "proxies:")?;
//         for proxy in proxies {
//             writeln!(
//                 output,
//                 "{:indent$}{}",
//                 "",
//                 format_args!("- {}", Self::render_proxy(proxy)?),
//                 indent = INDENT,
//             )?;
//         }
//         writeln!(output)?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> Result<String> {
//         let mut output = String::new();
//         writeln!(output, "proxy-groups:")?;
//         for group in proxy_groups {
//             writeln!(
//                 output,
//                 "{:indent$}{}",
//                 "",
//                 format_args!("- {}", Self::render_proxy_group(group)?),
//                 indent = INDENT,
//             )?;
//         }
//         writeln!(output)?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_rule_providers(rule_providers: &[(String, RuleProvider)]) -> Result<String> {
//         let mut output = String::new();
//         writeln!(output, "rule-providers:")?;
//         for provider in rule_providers {
//             writeln!(
//                 output,
//                 "{:indent$}{}",
//                 "",
//                 format_args!("{}: {}", provider.0, Self::render_rule_provider(&provider.1)?),
//                 indent = INDENT,
//             )?;
//         }
//         writeln!(output)?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_rules(rules: &[Rule]) -> Result<String> {
//         let mut output = String::new();
//         writeln!(output, "rules:")?;
//         for rule in rules {
//             writeln!(
//                 output,
//                 "{:indent$}{}",
//                 "",
//                 format_args!(r#"- "{}""#, Self::render_rule(rule)?),
//                 indent = INDENT,
//             )?;
//         }
//         writeln!(output)?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_rules_with_payload(rules: &[Rule]) -> Result<String> {
//         let mut output = String::new();
//         writeln!(output, "payload:")?;
//         for rule in rules {
//             writeln!(
//                 output,
//                 "{:indent$}{}",
//                 "",
//                 format_args!(r#"- "{}""#, Self::render_rule_without_policy(rule)?),
//                 indent = INDENT,
//             )?;
//         }
//         writeln!(output)?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_rule(rule: &Rule) -> Result<String> {
//         let mut output = String::new();
//         write!(output, "{}", rule.rule_type.as_str())?;
//         if let Some(value) = &rule.value {
//             write!(output, ",{}", value)?;
//         }
//         write!(output, "{}", Self::render_policy(&rule.policy)?)?;
//         Ok(output)
//     }
//
//     pub fn render_rule_without_policy(rule: &Rule) -> Result<String> {
//         let mut output = String::new();
//         write!(output, "{}", rule.rule_type.as_str())?;
//         if let Some(value) = &rule.value {
//             write!(output, ",{}", value)?;
//         }
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_proxy(proxy: &Proxy) -> Result<String> {
//         let mut output = String::new();
//         write!(output, "{{")?;
//         write!(output, r#"name: "{}""#, &proxy.name)?;
//         write!(output, r#", type: "{}""#, &proxy.r#type)?;
//         write!(output, r#", server: "{}""#, &proxy.server)?;
//         write!(output, r#", port: {}"#, &proxy.port)?;
//         write!(output, r#", password: "{}""#, &proxy.password)?;
//         if let Some(udp) = &proxy.udp {
//             write!(output, r#", udp: {}"#, udp)?;
//         }
//         if let Some(tfo) = &proxy.tfo {
//             write!(output, r#", tfo: {}"#, tfo)?;
//         }
//         if let Some(cipher) = &proxy.cipher {
//             write!(output, r#", cipher: {}"#, cipher)?;
//         }
//         if let Some(sni) = &proxy.sni {
//             write!(output, r#", sni: "{}""#, sni)?;
//         }
//         if let Some(skip_cert_verify) = &proxy.skip_cert_verify {
//             write!(output, r#", skip-cert-verify: {}"#, skip_cert_verify)?;
//         }
//         write!(output, "}}")?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_proxy_group(group: &ProxyGroup) -> Result<String> {
//         let mut output = String::new();
//         write!(output, "{{")?;
//         write!(output, r#"name: "{}""#, group.name)?;
//         write!(output, r#", type: "{}""#, group.r#type.as_str())?;
//         write!(output, r#", proxies: [{}]"#, group.proxies.join(", "))?;
//         write!(output, "}}")?;
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_rule_provider(provider: &RuleProvider) -> Result<String> {
//         let mut output = String::new();
//         write!(output, "{{")?;
//         write!(output, r#"type: "{}""#, provider.r#type)?;
//         write!(output, r#", url: "{}""#, provider.url)?;
//         write!(output, r#", path: "{}""#, provider.path)?;
//         write!(output, r#", interval: {}"#, provider.interval)?;
//         write!(output, r#", size-limit: {}"#, provider.size_limit)?;
//         write!(output, r#", format: "{}""#, provider.format)?;
//         write!(output, r#", behavior: "{}""#, provider.behavior)?;
//         write!(output, "}}")?;
//         Ok(output)
//     }
//
//     pub fn render_policy(policy: &Policy) -> Result<String> {
//         let mut output = String::new();
//         write!(output, ",{}", policy.name)?;
//         if let Some(option) = &policy.option {
//             write!(output, ",{}", option)?;
//         }
//         Ok(output)
//     }
//
//     #[instrument(skip_all)]
//     pub fn render_provider_name_from_policy(policy: &Policy) -> Result<String> {
//         if policy == &Policy::subscription_policy() {
//             return Ok("Subscription".to_string());
//         }
//         let mut output = String::new();
//         write!(
//             output,
//             "{}_{}",
//             policy.name,
//             policy.option.as_ref().unwrap_or(&"policy".to_string())
//         )?;
//         Ok(output.replace("-", "_"))
//     }
// }

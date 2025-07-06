use crate::profile::clash_profile::ClashProfile;
use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::ProxyGroup;
use crate::profile::core::rule::Rule;
use crate::profile::core::rule_provider::RuleProvider;
use crate::profile::renderer::Result;
use std::fmt::Write;
use tracing::instrument;

pub const INDENT: usize = 4;

pub struct ClashRenderer;

impl ClashRenderer {
    #[instrument(skip_all)]
    pub fn render_profile(profile: &ClashProfile) -> Result<String> {
        let ClashProfile {
            proxies,
            proxy_groups,
            rules,
            rule_providers,
            ..
        } = &profile;
        let lines = [
            Self::render_general(profile)?,
            Self::render_proxies(proxies)?,
            Self::render_proxy_groups(proxy_groups)?,
            Self::render_rule_providers(rule_providers)?,
            Self::render_rules(rules)?,
        ]
        .join("\n");
        Ok(lines)
    }

    #[instrument(skip_all)]
    pub fn render_general(profile: &ClashProfile) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "port: {}", profile.port)?;
        writeln!(&mut output, "socks-port: {}", profile.socks_port)?;
        writeln!(&mut output, "redir-port: {}", profile.redir_port)?;
        writeln!(&mut output, "allow-lan: {}", profile.allow_lan)?;
        writeln!(&mut output, "mode: {}", profile.mode)?;
        writeln!(&mut output, "log-level: {}", profile.log_level)?;
        writeln!(&mut output, r#"external-controller: "{}""#, profile.external_controller)?;
        writeln!(&mut output, r#"secret: "{}""#, profile.secret)?;
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_proxies(proxies: &[Proxy]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "proxies:")?;
        for proxy in proxies {
            writeln!(
                &mut output,
                "{:indent$}{}",
                "",
                format_args!("- {}", Self::render_proxy(proxy)?),
                indent = INDENT,
            )?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "proxy-groups:")?;
        for group in proxy_groups {
            writeln!(
                &mut output,
                "{:indent$}{}",
                "",
                format_args!("- {}", Self::render_proxy_group(group)?),
                indent = INDENT,
            )?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_rule_providers(rule_providers: &[(String, RuleProvider)]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "rule-providers:")?;
        for provider in rule_providers {
            writeln!(
                &mut output,
                "{:indent$}{}",
                "",
                format_args!("{}: {}", provider.0, Self::render_rule_provider(&provider.1)?),
                indent = INDENT,
            )?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_rules(rules: &[Rule]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "rules:")?;
        for rule in rules {
            writeln!(
                &mut output,
                "{:indent$}{}",
                "",
                format_args!(r#"- "{}""#, Self::render_rule(rule)?),
                indent = INDENT,
            )?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_rules_with_payload(rules: &[Rule]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "payload:")?;
        for rule in rules {
            writeln!(
                &mut output,
                "{:indent$}{}",
                "",
                format_args!(r#"- "{}""#, Self::render_rule_without_policy(rule)?),
                indent = INDENT,
            )?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_rule(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(output, ",{}", value)?;
        }
        write!(output, "{}", Self::render_policy(&rule.policy)?)?;
        Ok(output)
    }

    pub fn render_rule_without_policy(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(output, ",{}", value)?;
        }
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_proxy(proxy: &Proxy) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{")?;
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
        write!(output, "}}")?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_proxy_group(group: &ProxyGroup) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{")?;
        write!(output, r#"name: "{}""#, group.name)?;
        write!(output, r#", type: "{}""#, group.r#type.as_str())?;
        write!(output, r#", proxies: [{}]"#, group.proxies.join(", "))?;
        write!(output, "}}")?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_rule_provider(provider: &RuleProvider) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{")?;
        write!(output, r#"type: "{}""#, provider.r#type)?;
        write!(output, r#", url: "{}""#, provider.url)?;
        write!(output, r#", path: "{}""#, provider.path)?;
        write!(output, r#", interval: {}"#, provider.interval)?;
        write!(output, r#", size-limit: {}"#, provider.size_limit)?;
        write!(output, r#", format: "{}""#, provider.format)?;
        write!(output, r#", behavior: "{}""#, provider.behavior)?;
        write!(output, "}}")?;
        Ok(output)
    }

    pub fn render_policy(policy: &Policy) -> Result<String> {
        let mut output = String::new();
        write!(&mut output, ",{}", policy.name)?;
        if let Some(option) = &policy.option {
            write!(&mut output, ",{}", option)?;
        }
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_provider_name_from_policy(policy: &Policy) -> Result<String> {
        if policy == &Policy::subscription_policy() {
            return Ok("Subscription".to_string());
        }
        let mut output = String::new();
        write!(
            output,
            "{}_{}",
            policy.name,
            policy.option.as_ref().unwrap_or(&"policy".to_string())
        )?;
        Ok(output.replace("-", "_"))
    }
}

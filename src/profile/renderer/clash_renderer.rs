use crate::profile::clash_profile::ClashProfile;
use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::ProxyGroup;
use crate::profile::core::rule::Rule;
use crate::profile::core::rule_provider::RuleProvider;
use std::fmt::Write;
use tracing::instrument;

pub const INDENT: usize = 4;

pub struct ClashRenderer;

impl ClashRenderer {
    #[instrument(skip_all)]
    pub fn render_profile(output: &mut String, profile: &ClashProfile) -> color_eyre::Result<()> {
        let ClashProfile {
            proxies,
            proxy_groups,
            rules,
            rule_providers,
            ..
        } = &profile;
        Self::render_general(output, profile)?;
        Self::render_proxies(output, proxies)?;
        Self::render_proxy_groups(output, proxy_groups)?;
        Self::render_rule_providers(output, rule_providers)?;
        Self::render_rules(output, rules)?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn render_general(output: &mut String, profile: &ClashProfile) -> color_eyre::Result<()> {
        writeln!(output, "port: {}", profile.port)?;
        writeln!(output, "socks-port: {}", profile.socks_port)?;
        writeln!(output, "redir-port: {}", profile.redir_port)?;
        writeln!(output, "allow-lan: {}", profile.allow_lan)?;
        writeln!(output, "mode: {}", profile.mode)?;
        writeln!(output, "log-level: {}", profile.log_level)?;
        writeln!(output, r#"external-controller: "{}""#, profile.external_controller)?;
        writeln!(output, r#"secret: "{}""#, profile.secret)?;
        writeln!(output)?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn render_proxies(output: &mut String, proxies: &[Proxy]) -> color_eyre::Result<()> {
        writeln!(output, "proxies:")?;
        for proxy in proxies {
            writeln!(
                output,
                "{:indent$}{}",
                "",
                format_args!("- {}", Self::render_proxy(proxy)?),
                indent = INDENT,
            )?;
        }
        writeln!(output)?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn render_proxy_groups(output: &mut String, proxy_groups: &[ProxyGroup]) -> color_eyre::Result<()> {
        writeln!(output, "proxy-groups:")?;
        for group in proxy_groups {
            writeln!(
                output,
                "{:indent$}{}",
                "",
                format_args!("- {}", Self::render_proxy_group(group)?),
                indent = INDENT,
            )?;
        }
        writeln!(output)?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn render_rule_providers(
        output: &mut String,
        rule_providers: &[(String, RuleProvider)],
    ) -> color_eyre::Result<()> {
        writeln!(output, "rule-providers:")?;
        for provider in rule_providers {
            writeln!(
                output,
                "{:indent$}{}",
                "",
                format_args!("{}: {}", provider.0, Self::render_rule_provider(&provider.1)?),
                indent = INDENT,
            )?;
        }
        writeln!(output)?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn render_rules(output: &mut String, rules: &[Rule]) -> color_eyre::Result<()> {
        writeln!(output, "rules:")?;
        for rule in rules {
            writeln!(
                output,
                "{:indent$}{}",
                "",
                format_args!(r#"- "{}""#, Self::render_rule(rule)?),
                indent = INDENT,
            )?;
        }
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn render_rule_provider_payload(output: &mut String, rules: &[&Rule]) -> color_eyre::Result<()> {
        writeln!(output, "payload:")?;
        for rule in rules {
            writeln!(
                output,
                "{:indent$}{}",
                "",
                format_args!(r#"- "{}""#, Self::render_rule(rule)?),
                indent = INDENT,
            )?;
        }
        Ok(())
    }

    #[instrument(skip_all)]
    fn render_proxy(proxy: &Proxy) -> color_eyre::Result<String> {
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
    fn render_proxy_group(group: &ProxyGroup) -> color_eyre::Result<String> {
        let mut output = String::new();
        write!(output, "{{")?;
        write!(output, r#"name: "{}""#, group.name)?;
        write!(output, r#", type: "{}""#, group.r#type.as_str())?;
        write!(output, r#", proxies: [{}]"#, group.proxies.join(", "))?;
        write!(output, "}}")?;
        Ok(output)
    }

    #[instrument(skip_all)]
    fn render_rule_provider(provider: &RuleProvider) -> color_eyre::Result<String> {
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

    #[instrument(skip_all)]
    fn render_rule(rule: &Rule) -> color_eyre::Result<String> {
        let mut output = String::new();
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(output, ",{}", value)?;
        }
        write!(output, ",{}", rule.policy.name)?;
        if let Some(option) = &rule.policy.option {
            write!(output, ",{}", option)?;
        }
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_policy_for_provider(policy: &Policy) -> String {
        let mut output = String::new();
        write!(output, "{}", policy.name).expect("无法写入 Clash 规则集名称");
        if let Some(option) = policy.option.as_ref() {
            write!(output, "-{}", option).expect("无法写入 Clash 规则集选项");
        }
        output
    }
}

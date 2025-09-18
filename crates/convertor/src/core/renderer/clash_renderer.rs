use crate::config::client_config::ProxyClient;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::profile::rule_provider::RuleProvider;
use crate::core::renderer::{INDENT, Renderer};
use crate::error::RenderError;
use std::fmt::Write;
use tracing::instrument;

type Result<T> = core::result::Result<T, RenderError>;

pub struct ClashRenderer;

impl Renderer for ClashRenderer {
    type PROFILE = ClashProfile;

    fn client() -> ProxyClient {
        ProxyClient::Clash
    }

    #[instrument(skip_all)]
    fn render_profile(profile: &Self::PROFILE) -> Result<String> {
        let mut output = String::new();
        writeln!(output, "{}", Self::render_general(profile)?)?;

        let proxies = Self::render_proxies(&profile.proxies)?;
        writeln!(output, "proxies:")?;
        writeln!(output, "{proxies}")?;

        let proxy_groups = Self::render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output, "proxy-groups:")?;
        writeln!(output, "{proxy_groups}")?;

        let rule_providers = Self::render_rule_providers(&profile.rule_providers)?;
        writeln!(output, "rule-providers:")?;
        writeln!(output, "{rule_providers}")?;

        let rules = Self::render_rules(&profile.rules)?;
        writeln!(output, "rules:")?;
        writeln!(output, "{rules}")?;

        Ok(output)
    }

    #[instrument(skip_all)]
    fn render_general(profile: &Self::PROFILE) -> Result<String> {
        let mut output = String::new();
        writeln!(output, "port: {}", profile.port)?;
        writeln!(output, "socks-port: {}", profile.socks_port)?;
        writeln!(output, "redir-port: {}", profile.redir_port)?;
        writeln!(output, "allow-lan: {}", profile.allow_lan)?;
        writeln!(output, "mode: {}", profile.mode)?;
        writeln!(output, "log-level: {}", profile.log_level)?;
        writeln!(output, r#"external-controller: {}"#, profile.external_controller)?;
        writeln!(output, r#"external-ui: {}"#, profile.external_ui)?;
        if let Some(secret) = &profile.secret {
            writeln!(output, r#"secret: "{secret}""#)?;
        }
        Ok(output)
    }

    fn render_proxy(proxy: &Proxy) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{ ")?;
        write!(output, r#"name: "{}""#, &proxy.name)?;
        write!(output, r#", type: "{}""#, &proxy.r#type)?;
        write!(output, r#", server: "{}""#, &proxy.server)?;
        write!(output, r#", port: {}"#, &proxy.port)?;
        write!(output, r#", password: "{}""#, &proxy.password)?;
        if let Some(udp) = &proxy.udp {
            write!(output, r#", udp: {udp}"#)?;
        }
        if let Some(tfo) = &proxy.tfo {
            write!(output, r#", tfo: {tfo}"#)?;
        }
        if let Some(cipher) = &proxy.cipher {
            write!(output, r#", cipher: {cipher}"#)?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, r#", sni: "{sni}""#)?;
        }
        if let Some(skip_cert_verify) = &proxy.skip_cert_verify {
            write!(output, r#", skip-cert-verify: {skip_cert_verify}"#)?;
        }
        write!(output, " }}")?;
        Ok(output)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{ ")?;
        write!(output, r#"name: "{}""#, proxy_group.name)?;
        write!(output, r#", type: "{}""#, proxy_group.r#type.as_str())?;
        write!(output, r#", proxies: [ {} ]"#, proxy_group.proxies.join(", "))?;
        write!(output, " }}")?;
        Ok(output)
    }

    fn render_rule(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(output, ",{value}")?;
        }
        write!(output, ",{}", Self::render_policy(&rule.policy)?)?;
        Ok(output)
    }

    fn render_rule_for_provider(rule: &Rule) -> Result<String> {
        Self::render_rule(rule)
    }

    fn render_provider_rule(rule: &ProviderRule) -> Result<String> {
        Ok(format!("{},{}", rule.rule_type.as_str(), rule.value))
    }

    #[instrument(skip_all)]
    fn render_rule_providers(rule_providers: &[(String, RuleProvider)]) -> Result<String> {
        let output = rule_providers
            .iter()
            .map(Self::render_rule_provider)
            .map(|line| line.map(|line| format!("{:indent$}{}", "", line, indent = INDENT)))
            .collect::<Result<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    fn render_rule_provider(rule_provider: &(String, RuleProvider)) -> Result<String> {
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

    fn render_provider_name_for_policy(policy: &Policy) -> String {
        let mut output = if policy.is_subscription {
            "Subscription".to_string()
        } else {
            policy.name.clone()
        };
        match &policy.option {
            Some(option) => {
                output.push('_');
                output.push_str(option.replace('-', "_").as_str());
            }
            None => {
                output.push_str("_policy");
            }
        }
        output
    }
}

use crate::config::client_config::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::profile::rule_provider::RuleProvider;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::result::RenderResult;
use std::fmt::Write;
use tracing::instrument;

pub const SURGE_RULE_PROVIDER_COMMENT_START: &str = "# Rule Provider from convertor";
pub const SURGE_RULE_PROVIDER_COMMENT_END: &str = "# End of Rule Provider";

pub struct SurgeRenderer;

impl Renderer for SurgeRenderer {
    type PROFILE = SurgeProfile;

    fn client() -> ProxyClient {
        ProxyClient::Surge
    }

    fn render_profile(profile: &Self::PROFILE) -> RenderResult<String> {
        let mut output = String::new();

        let header = Self::render_header(profile)?;
        writeln!(output, "{}", header.trim())?;
        writeln!(output)?;

        let general = Self::render_general(profile)?;
        writeln!(output, "[General]")?;
        writeln!(output, "{}", general.trim())?;
        writeln!(output)?;

        let proxies = Self::render_proxies(&profile.proxies)?;
        writeln!(output, "[Proxy]")?;
        writeln!(output, "{}", proxies.trim())?;
        writeln!(output)?;

        let proxy_groups = Self::render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output, "[Proxy Group]")?;
        writeln!(output, "{}", proxy_groups.trim())?;
        writeln!(output)?;

        let rules = Self::render_rules(&profile.rules)?;
        writeln!(output, "[Rule]")?;
        writeln!(output, "{}", rules.trim())?;
        writeln!(output)?;

        let url_rewrite = Self::render_url_rewrite(&profile.url_rewrite)?;
        writeln!(output, "[URL Rewrite]")?;
        writeln!(output, "{}", url_rewrite.trim())?;
        writeln!(output)?;

        let misc = Self::render_misc(&profile.misc)?;
        if !misc.trim().is_empty() {
            writeln!(output, "{}", misc.trim())?;
        }

        Ok(output)
    }

    fn render_general(profile: &Self::PROFILE) -> RenderResult<String> {
        Self::render_lines(&profile.general, |line| Ok(line.clone()))
    }

    fn render_proxy(proxy: &Proxy) -> RenderResult<String> {
        let mut output = String::new();
        if let Some(comment) = &proxy.comment {
            writeln!(output, "{comment}")?;
        }
        write!(
            output,
            "{}={},{},{},password={}",
            proxy.name, proxy.r#type, proxy.server, proxy.port, proxy.password
        )?;
        if let Some(cipher) = &proxy.cipher {
            write!(output, ",encrypt-method={cipher}")?;
        }
        if let Some(udp) = proxy.udp {
            write!(output, ",udp-relay={udp}")?;
        }
        if let Some(tfo) = proxy.tfo {
            write!(output, ",tfo={tfo}")?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, ",sni={sni}")?;
        }
        if let Some(skip_cert_verify) = proxy.skip_cert_verify {
            write!(output, ",skip-cert-verify={skip_cert_verify}")?;
        }
        Ok(output)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> RenderResult<String> {
        let mut output = String::new();
        if let Some(comment) = &proxy_group.comment {
            writeln!(output, "{comment}")?;
        }
        write!(output, "{}={}", proxy_group.name, proxy_group.r#type.as_str())?;
        if !proxy_group.proxies.is_empty() {
            write!(output, ",{}", proxy_group.proxies.join(","))?;
        }
        Ok(output)
    }

    fn render_rule(rule: &Rule) -> RenderResult<String> {
        let mut output = String::new();
        if let Some(comment) = &rule.comment {
            writeln!(output, "{comment}")?;
        }
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(output, ",{value}")?;
        }
        write!(output, ",{}", Self::render_policy(&rule.policy)?)?;
        Ok(output)
    }

    fn render_rule_for_provider(rule: &Rule) -> RenderResult<String> {
        Ok(format!(
            "{},{},{}",
            rule.rule_type.as_str(),
            rule.value.as_ref().expect("规则集中的规则必须有 value"),
            Self::render_policy(&rule.policy)?,
        ))
    }

    fn render_provider_rule(rule: &ProviderRule) -> RenderResult<String> {
        let mut output = String::new();
        if let Some(comment) = &rule.comment {
            writeln!(output, "{comment}")?;
        }
        write!(output, "{},{}", rule.rule_type.as_str(), rule.value)?;
        Ok(output)
    }

    fn render_rule_providers(_: &[(String, RuleProvider)]) -> RenderResult<String> {
        todo!("SurgeRenderer 不会渲染 [RuleProvider]");
    }

    fn render_rule_provider(_: &(String, RuleProvider)) -> RenderResult<String> {
        todo!("SurgeRenderer 不会渲染 RuleProvider");
    }

    fn render_provider_name_for_policy(policy: &Policy) -> RenderResult<String> {
        let mut output = String::new();
        write!(output, "[")?;
        if policy.is_subscription {
            write!(output, "Subscription")?;
        } else {
            write!(output, "{}", policy.name)?;
        }
        if let Some(option) = policy.option.as_ref() {
            write!(output, ": {option}")?;
        }
        write!(output, "] by convertor/{}", env!("CARGO_PKG_VERSION"))?;
        Ok(output)
    }
}

impl SurgeRenderer {
    #[instrument(skip_all)]
    pub fn render_header(profile: &SurgeProfile) -> RenderResult<String> {
        Ok(profile.header.to_string())
    }

    #[instrument(skip_all)]
    pub fn render_url_rewrite(url_rewrite: &[String]) -> RenderResult<String> {
        Self::render_lines(url_rewrite, |line| Ok(line.clone()))
    }

    #[instrument(skip_all)]
    pub fn render_misc(misc: &[(String, Vec<String>)]) -> RenderResult<String> {
        let mut output = String::new();
        for (key, values) in misc {
            writeln!(output, "{key}")?;
            let lines = Self::render_lines(values, |value| Ok(value.clone()))?;
            writeln!(output, "{lines}")?;
        }
        Ok(output)
    }
}

#![deny(unused, unused_variables)]

use crate::client::Client;
use crate::core::profile::policy::Policy;
use crate::core::profile::profile::Profile;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::profile::rule_provider::RuleProvider;
use crate::core::result::RenderResult;
use std::fmt::Write;
use tracing::instrument;

pub mod surge_renderer;
pub mod clash_renderer;

pub const INDENT: usize = 4;

pub trait Renderer {
    type PROFILE: Profile;

    fn client() -> Client;

    fn render_profile(profile: &Self::PROFILE) -> RenderResult<String>;

    fn render_general(profile: &Self::PROFILE) -> RenderResult<String>;

    #[instrument(skip_all)]
    fn render_proxies(proxies: &[Proxy]) -> RenderResult<String> {
        Self::render_lines(proxies, Self::render_proxy)
    }

    fn render_proxy(proxy: &Proxy) -> RenderResult<String>;

    #[instrument(skip_all)]
    fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> RenderResult<String> {
        Self::render_lines(proxy_groups, Self::render_proxy_group)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> RenderResult<String>;

    #[instrument(skip_all)]
    fn render_rules(rules: &[Rule]) -> RenderResult<String> {
        Self::render_lines(rules, Self::render_rule)
    }

    fn render_rule(rule: &Rule) -> RenderResult<String>;

    /// 适用于渲染规则集类型的规则，不包含注释
    fn render_rule_for_provider(rule: &Rule) -> RenderResult<String>;

    #[instrument(skip_all)]
    fn render_provider_rules(rules: &[ProviderRule]) -> RenderResult<String> {
        let mut output = String::new();
        match Self::client() {
            Client::Surge => {
                writeln!(output, "{}", Self::render_lines(rules, Self::render_provider_rule)?)?;
            }
            Client::Clash => {
                writeln!(output, "payload:")?;
                writeln!(output, "{}", Self::render_lines(rules, Self::render_provider_rule)?)?;
            }
        }
        Ok(output)
    }

    fn render_provider_rule(rule: &ProviderRule) -> RenderResult<String>;

    fn render_policy(policy: &Policy) -> RenderResult<String> {
        let mut output = String::new();
        write!(output, "{}", policy.name)?;
        if let Some(option) = &policy.option {
            write!(output, ",{}", option)?;
        }
        Ok(output)
    }

    fn render_rule_providers(rule_providers: &[(String, RuleProvider)]) -> RenderResult<String>;

    fn render_rule_provider(rule_provider: &(String, RuleProvider)) -> RenderResult<String>;

    fn render_provider_name_for_policy(policy: &Policy) -> RenderResult<String>;

    fn render_lines<T, F>(lines: impl IntoIterator<Item = T>, map: F) -> RenderResult<String>
    where
        F: FnMut(T) -> RenderResult<String>,
    {
        let output = lines
            .into_iter()
            .map(map)
            .map(|line| match Self::client() {
                Client::Surge => line,
                Client::Clash => line.map(Self::indent_line),
            })
            .collect::<RenderResult<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    fn indent_line(line: String) -> String {
        format!("{:indent$}{}", "", format_args!("- {}", line), indent = INDENT)
    }
}

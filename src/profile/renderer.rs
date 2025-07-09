#![deny(unused, unused_variables)]

use crate::client::Client;
use crate::profile::core::policy::Policy;
use crate::profile::core::profile::Profile;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::ProxyGroup;
use crate::profile::core::rule::{ProviderRule, Rule};
use crate::profile::core::rule_provider::RuleProvider;
use crate::profile::error::RenderError;
use std::fmt::Write;
use tracing::instrument;

pub mod surge_renderer;
pub mod clash_renderer;

pub const INDENT: usize = 4;

pub type Result<T> = core::result::Result<T, RenderError>;

pub trait Renderer {
    type PROFILE: Profile;

    fn client() -> Client;

    fn render_profile(profile: &Self::PROFILE) -> Result<String>;

    fn render_general(profile: &Self::PROFILE) -> Result<String>;

    #[instrument(skip_all)]
    fn render_proxies(proxies: &[Proxy]) -> Result<String> {
        Self::render_lines(proxies, Self::render_proxy)
    }

    fn render_proxy(proxy: &Proxy) -> Result<String>;

    #[instrument(skip_all)]
    fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> Result<String> {
        Self::render_lines(proxy_groups, Self::render_proxy_group)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String>;

    #[instrument(skip_all)]
    fn render_rules(rules: &[Rule]) -> Result<String> {
        Self::render_lines(rules, Self::render_rule)
    }

    fn render_rule(rule: &Rule) -> Result<String>;

    // #[instrument(skip_all)]
    // fn render_rules_for_provider(rules: &[Rule]) -> Result<String> {
    //     Ok(Self::render_lines(rules, Self::render_rule_for_provider)?)
    // }

    /// 适用于渲染规则集类型的规则，不包含注释
    fn render_rule_for_provider(rule: &Rule) -> Result<String>;

    #[instrument(skip_all)]
    fn render_provider_rules(rules: &[ProviderRule]) -> Result<String> {
        let mut output = String::new();
        match Self::client() {
            Client::Surge => {
                writeln!(output, "{}", Self::render_lines(rules, Self::render_provider_rule)?)?;
            }
            Client::Clash => {
                writeln!(output, "payloads:")?;
                writeln!(output, "{}", Self::render_lines(rules, Self::render_provider_rule)?)?;
            }
        }
        Ok(output)
    }

    fn render_provider_rule(rule: &ProviderRule) -> Result<String>;

    fn render_policy(policy: &Policy) -> Result<String> {
        let mut output = String::new();
        write!(output, "{}", policy.name)?;
        if let Some(option) = &policy.option {
            write!(output, ",{}", option)?;
        }
        Ok(output)
    }

    fn render_rule_providers(rule_providers: &[(String, RuleProvider)]) -> Result<String>;

    fn render_rule_provider(rule_provider: &(String, RuleProvider)) -> Result<String>;

    fn render_provider_name_for_policy(policy: &Policy) -> Result<String>;

    fn render_lines<T, F>(lines: impl IntoIterator<Item = T>, map: F) -> Result<String>
    where
        F: FnMut(T) -> Result<String>,
    {
        let output = lines
            .into_iter()
            .map(map)
            .map(|line| match Self::client() {
                Client::Surge => line,
                Client::Clash => line.map(Self::indent_line),
            })
            .collect::<Result<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    fn indent_line(line: String) -> String {
        format!("{:indent$}{}", "", format_args!("- {}", line), indent = INDENT)
    }
}

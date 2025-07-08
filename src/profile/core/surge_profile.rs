use crate::client::Client;
use crate::config::surge_config::SurgeConfig;
use crate::profile::core::policy::Policy;
use crate::profile::core::profile::Profile;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::ProxyGroup;
use crate::profile::core::rule::{ProviderRule, Rule};
use crate::profile::parser::surge_parser::SurgeParser;
use crate::profile::renderer::Renderer;
use crate::profile::renderer::surge_renderer::SurgeRenderer;
use crate::subscription::url_builder::UrlBuilder;
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug)]
pub struct SurgeProfile {
    pub header: String,
    pub general: Vec<String>,
    pub proxies: Vec<Proxy>,
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<Rule>,
    pub url_rewrite: Vec<String>,
    pub misc: Vec<(String, Vec<String>)>,
    pub policy_of_rules: HashMap<Policy, Vec<ProviderRule>>,
}

impl Profile for SurgeProfile {
    type PROFILE = SurgeProfile;

    fn proxies(&self) -> &[Proxy] {
        &self.proxies
    }

    fn proxies_mut(&mut self) -> &mut Vec<Proxy> {
        &mut self.proxies
    }

    fn proxy_groups(&self) -> &[ProxyGroup] {
        &self.proxy_groups
    }

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup> {
        &mut self.proxy_groups
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    fn rules_mut(&mut self) -> &mut Vec<Rule> {
        &mut self.rules
    }

    fn policy_of_rules(&self) -> &HashMap<Policy, Vec<ProviderRule>> {
        &self.policy_of_rules
    }

    fn policy_of_rules_mut(&mut self) -> &mut HashMap<Policy, Vec<ProviderRule>> {
        &mut self.policy_of_rules
    }

    #[instrument(skip_all)]
    fn parse(content: String) -> color_eyre::Result<Self::PROFILE> {
        Ok(SurgeParser::parse_profile(content)?)
    }

    #[instrument(skip_all)]
    fn optimize(
        &mut self,
        url_builder: &UrlBuilder,
        _: Option<String>,
        _: Option<impl AsRef<str>>,
    ) -> color_eyre::Result<()> {
        self.replace_header(url_builder)?;
        self.optimize_proxies()?;
        self.optimize_rules(url_builder)?;
        Ok(())
    }

    #[instrument(skip_all)]
    fn append_rule_provider(&mut self, policy: &Policy, url_builder: &UrlBuilder) -> color_eyre::Result<()> {
        let name = SurgeRenderer::render_provider_name_for_policy(policy)?;
        let url = url_builder.build_rule_provider_url(Client::Surge, policy)?;
        let rule = Rule::surge_rule_provider(policy, name, url)?;
        self.rules.push(rule);
        Ok(())
    }
}

impl SurgeProfile {
    fn replace_header(&mut self, url_builder: &UrlBuilder) -> color_eyre::Result<()> {
        let url = url_builder.build_convertor_url(Client::Surge)?;
        self.header = SurgeConfig::build_managed_config_header(url);
        Ok(())
    }
}

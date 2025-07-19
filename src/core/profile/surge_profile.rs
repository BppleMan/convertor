use crate::common::proxy_client::ProxyClient;
use crate::common::url::ConvertorUrl;
use crate::core::parser::surge_parser::SurgeParser;
use crate::core::profile::Profile;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::result::ParseResult;
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone)]
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

    fn client() -> ProxyClient {
        ProxyClient::Surge
    }

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
    fn parse(content: String) -> ParseResult<Self::PROFILE> {
        SurgeParser::parse_profile(content)
    }

    #[instrument(skip_all)]
    fn optimize(&mut self, url: &ConvertorUrl) -> ParseResult<()> {
        self.replace_header(url)?;
        self.optimize_proxies()?;
        self.optimize_rules(url)?;
        Ok(())
    }

    #[instrument(skip_all)]
    fn append_rule_provider(&mut self, policy: &Policy, url: &ConvertorUrl) -> ParseResult<()> {
        let name = SurgeRenderer::render_provider_name_for_policy(policy)?;
        let url = url.build_rule_provider_url(policy)?;
        let rule = Rule::surge_rule_provider(policy, name, url);
        self.rules.push(rule);
        Ok(())
    }
}

impl SurgeProfile {
    #[instrument(skip_all)]
    fn replace_header(&mut self, url: &ConvertorUrl) -> ParseResult<()> {
        self.header = Self::build_managed_config_header(url)?;
        Ok(())
    }

    pub fn build_managed_config_header(convertor_url: &ConvertorUrl) -> ParseResult<String> {
        let url = convertor_url.build_sub_url()?;
        let header = format!(
            "#!MANAGED-CONFIG {} interval={} strict={}",
            url.as_str(),
            convertor_url.interval,
            convertor_url.strict
        );
        Ok(header)
    }
}

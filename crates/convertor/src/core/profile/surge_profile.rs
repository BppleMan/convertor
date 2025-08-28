use crate::config::client_config::ProxyClient;
use crate::core::parser::surge_parser::SurgeParser;
use crate::core::profile::Profile;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::result::ParseResult;
use crate::url::convertor_url::ConvertorUrlType;
use crate::url::url_builder::UrlBuilder;
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
    fn convert(&mut self, url_builder: &UrlBuilder) -> ParseResult<()> {
        self.replace_header(url_builder)?;
        self.optimize_proxies()?;
        self.optimize_rules(url_builder)?;
        Ok(())
    }

    #[instrument(skip_all)]
    fn append_rule_provider(&mut self, url_builder: &UrlBuilder, policy: &Policy) -> ParseResult<()> {
        let name = SurgeRenderer::render_provider_name_for_policy(policy);
        let url = url_builder.build_rule_provider_url(policy)?;
        let rule = Rule::surge_rule_provider(policy, name, url);
        self.rules.push(rule);
        Ok(())
    }
}

impl SurgeProfile {
    #[instrument(skip_all)]
    fn replace_header(&mut self, url_builder: &UrlBuilder) -> ParseResult<()> {
        self.header = url_builder.build_surge_header(ConvertorUrlType::Profile)?.to_string();
        Ok(())
    }
}

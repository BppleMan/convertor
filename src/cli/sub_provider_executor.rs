use crate::api::SubProviderWrapper;
use crate::cli::sub_provider_executor::result::{SubProviderExecutorLink, SubProviderExecutorResult};
use crate::common::config::ConvertorConfig;
use crate::common::config::proxy_client::{ClashConfig, ProxyClient, ProxyClientConfig, SurgeConfig};
use crate::common::config::sub_provider::SubProvider;
use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::extract_policies_for_rule_provider;
use crate::core::profile::policy::Policy;
use crate::core::profile::rule::Rule;
use crate::core::profile::surge_header::{SurgeHeader, SurgeHeaderType};
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::renderer::surge_renderer::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START};
use crate::core::result::ParseResult;
use crate::core::url_builder::{HostPort, UrlBuilder};
use clap::{Args, Subcommand};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use url::Url;

pub mod result;

#[derive(Default, Debug, Clone, Args)]
pub struct SubProviderCmd {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: ProxyClient,

    /// 订阅提供商
    #[arg(value_enum, default_value_t = SubProvider::BosLife)]
    pub provider: SubProvider,

    /// convertor 所在服务器的地址
    /// 格式为 `http://ip:port`
    /// 未指定时，使用配置文件中的默认值
    #[arg(short, long)]
    pub server: Option<Url>,

    /// 订阅更新的间隔时间，单位为秒
    /// 未指定时，使用配置文件中的默认值
    #[arg(short, long)]
    pub interval: Option<u64>,

    /// 是否严格模式
    /// 如果开启，订阅转换器将严格按照配置进行转换
    #[arg(short = 'S', long)]
    pub strict: Option<bool>,

    /// 是否更新本地订阅文件
    #[arg(short, long, default_value_t = false)]
    pub update: bool,

    #[command(subcommand)]
    pub url_source: Option<UrlSource>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum UrlSource {
    /// 使用 订阅提供商API 获取最新订阅链接
    Get,

    /// 使用重置的原始订阅链接
    Reset,

    /// 解码 订阅提供商 的原始订阅链接
    Raw { sub_url: Option<Url> },

    /// 解码 convertor 的完整订阅链接
    Decode { profile_url: Url },
}

pub struct SubProviderExecutor {
    pub config: ConvertorConfig,
    pub api_map: HashMap<SubProvider, SubProviderWrapper>,
}

#[allow(clippy::large_enum_variant)]
enum ClientProfile {
    Surge,
    Clash(ClashProfile),
}

impl SubProviderExecutor {
    pub fn new(config: ConvertorConfig, api_map: HashMap<SubProvider, SubProviderWrapper>) -> Self {
        Self { config, api_map }
    }

    pub async fn execute(&mut self, cmd: SubProviderCmd) -> Result<(UrlBuilder, SubProviderExecutorResult)> {
        let client = cmd.client;
        let sub_provider = cmd.provider;
        let url_builder = self.create_url_builder(&cmd).await?;
        let raw_profile_content = match self.api_map.get_mut(&sub_provider) {
            Some(api) => {
                api.set_sub_url(url_builder.sub_url.clone());
                api.get_raw_profile(client).await?
            }
            None => {
                return Err(eyre!("无法取得订阅供应商的 api 实现: {}", &sub_provider));
            }
        };
        let uni_sub_host_port = url_builder.sub_url.host_port()?;
        let (client_profile, policies) = match client {
            ProxyClient::Surge => {
                let mut raw_profile = SurgeProfile::parse(raw_profile_content)?;
                raw_profile.convert(&url_builder)?;
                let mut policies: Vec<Policy> = raw_profile.policy_of_rules.keys().cloned().collect();
                policies.sort();
                (ClientProfile::Surge, policies)
            }
            ProxyClient::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let policies = extract_policies_for_rule_provider(&raw_profile.rules, uni_sub_host_port);
                (ClientProfile::Clash(raw_profile), policies)
            }
        };

        let raw_sub_url = url_builder.build_raw_url();
        let profile_url = url_builder.build_profile_url();
        let raw_profile_url = url_builder.build_raw_profile_url();
        let sub_logs_url = url_builder.build_sub_logs_url(1, 20)?;

        let raw_link = SubProviderExecutorLink::raw((&raw_sub_url).into());
        let profile_link = SubProviderExecutorLink::profile((&profile_url).into());
        let raw_profile_link = SubProviderExecutorLink::raw_profile((&raw_profile_url).into());
        let logs_link = SubProviderExecutorLink::logs((&sub_logs_url).into());
        let rule_provider_links = policies
            .iter()
            .map(|policy| {
                let name = match client {
                    ProxyClient::Surge => SurgeRenderer::render_provider_name_for_policy(policy)?,
                    ProxyClient::Clash => ClashRenderer::render_provider_name_for_policy(policy)?,
                };
                let url = url_builder.build_rule_provider_url(policy);
                Ok(SubProviderExecutorLink::rule_provider(name, (&url).into()))
            })
            .collect::<Result<Vec<_>>>()?;
        let result = SubProviderExecutorResult {
            raw_link,
            raw_profile_link,
            profile_link,
            logs_link,
            rule_provider_links,
        };

        // 副作用逻辑后置，主流程只负责数据流
        if cmd.update {
            match client_profile {
                ClientProfile::Surge => {
                    self.update_surge_config(&url_builder, &result.logs_link, &policies)
                        .await?;
                }
                ClientProfile::Clash(profile) => {
                    self.update_clash_config(&url_builder, profile).await?;
                }
            }
        }
        Ok((url_builder, result))
    }

    pub fn post_execute(&self, _url_builder: UrlBuilder, result: SubProviderExecutorResult) {
        println!("{result}");
    }

    async fn create_url_builder(&self, cmd: &SubProviderCmd) -> Result<UrlBuilder> {
        let SubProviderCmd {
            client,
            provider,
            server,
            interval,
            update: _update,
            strict,
            url_source,
        } = cmd;
        let mut url_builder = match url_source {
            None => self.config.create_url_builder(*client, *provider)?,
            Some(UrlSource::Get) => {
                let sub_url = self
                    .api_map
                    .get(provider)
                    .ok_or(eyre!("无法取得订阅供应商的 api 实现: {}", &provider))?
                    .get_sub_url()
                    .await?;
                let mut url_builder = self.config.create_url_builder(*client, *provider)?;
                url_builder.sub_url = sub_url;
                url_builder
            }
            Some(UrlSource::Reset) => {
                let sub_url = self
                    .api_map
                    .get(provider)
                    .ok_or(eyre!("无法取得订阅供应商的 api 实现: {}", &provider))?
                    .reset_sub_url()
                    .await?;
                let mut url_builder = self.config.create_url_builder(*client, *provider)?;
                url_builder.sub_url = sub_url;
                url_builder
            }
            Some(UrlSource::Raw { sub_url }) => {
                let mut url_builder = self.config.create_url_builder(*client, *provider)?;
                url_builder.sub_url = match (sub_url, self.config.providers.get(provider)) {
                    (Some(sub_url), _) => sub_url.clone(),
                    (None, Some(config)) => config.sub_url().clone(),
                    _ => {
                        return Err(eyre!("未找到订阅提供商的原始订阅链接，请检查参数是否完整"));
                    }
                };
                url_builder
            }
            Some(UrlSource::Decode { profile_url }) => UrlBuilder::parse_from_url(profile_url, &self.config.secret)?,
        };

        if let Some(server) = server {
            url_builder.server = server.clone();
        }
        if let Some(interval) = interval {
            url_builder.interval = *interval;
        }
        if let Some(strict) = strict {
            url_builder.strict = *strict;
        }

        Ok(url_builder)
    }

    async fn update_surge_config(
        &self,
        url_builder: &UrlBuilder,
        sub_logs_link: &SubProviderExecutorLink,
        policies: &[Policy],
    ) -> Result<()> {
        if let Some(ProxyClientConfig::Surge(config)) = self.config.clients.get(&ProxyClient::Surge) {
            config.update_surge_config(url_builder, sub_logs_link, policies).await?;
        } else {
            eprintln!("{}", "Surge 配置未找到，请检查配置文件是否正确设置".red().bold());
        }
        Ok(())
    }

    async fn update_clash_config(&self, url_builder: &UrlBuilder, raw_profile: ClashProfile) -> Result<()> {
        if let Some(ProxyClientConfig::Clash(config)) = self.config.clients.get(&ProxyClient::Clash) {
            config
                .update_clash_config(url_builder, raw_profile, &self.config.secret)
                .await?;
        } else {
            eprintln!("{}", "Clash 配置未找到，请检查配置文件是否正确设置".red().bold());
        }
        Ok(())
    }
}

impl SurgeConfig {
    async fn update_surge_config(
        &self,
        url: &UrlBuilder,
        sub_logs_link: &SubProviderExecutorLink,
        policies: &[Policy],
    ) -> Result<()> {
        // 更新主订阅配置，即由 convertor 生成的订阅配置
        let header = url.build_managed_config_header(SurgeHeaderType::Profile);
        Self::update_conf(&self.main_profile_path(), header).await?;

        // 更新转发原始订阅配置，即由 convertor 生成的原始订阅配置
        if let Some(raw_profile_path) = self.raw_profile_path() {
            let header = url.build_managed_config_header(SurgeHeaderType::RawProfile);
            Self::update_conf(raw_profile_path, header).await?;
        }

        // 更新原始订阅配置，即由订阅提供商生成的订阅配置，如果存在的话
        if let Some(raw_sub_path) = self.raw_sub_path() {
            let header = url.build_managed_config_header(SurgeHeaderType::Raw);
            Self::update_conf(raw_sub_path, header).await?;
        }

        // 更新 rules.dconf 中的 RULE-SET 规则，规则提供者将从 policies 中生成 URL
        if let Some(rules_path) = self.rules_path() {
            self.update_surge_rule_providers(rules_path, url, policies).await?;
        }

        // 更新 subscription_logs.js 中的请求订阅日志的 URL
        if let Some(sub_logs_path) = self.sub_logs_path() {
            self.update_surge_sub_logs_url(sub_logs_path, sub_logs_link.url.as_str())
                .await?;
        }
        Ok(())
    }

    async fn update_surge_rule_providers(
        &self,
        rules_path: impl AsRef<Path>,
        url: &UrlBuilder,
        policies: &[Policy],
    ) -> Result<()> {
        let content = tokio::fs::read_to_string(&rules_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();

        let range_of_rule_providers = lines.iter().enumerate().fold(0..=0, |acc, (no, line)| {
            let mut start = *acc.start();
            let mut end = *acc.end();
            if line == SURGE_RULE_PROVIDER_COMMENT_START {
                start = no;
            } else if line == SURGE_RULE_PROVIDER_COMMENT_END {
                end = no;
            }
            start..=end
        });

        let provider_rules = policies
            .iter()
            .map(|policy| {
                let name = SurgeRenderer::render_provider_name_for_policy(policy)?;
                let url = url.build_rule_provider_url(policy);
                Ok(Rule::surge_rule_provider(policy, name, url))
            })
            .collect::<ParseResult<Vec<_>>>()?;
        let mut output = provider_rules
            .iter()
            .map(SurgeRenderer::render_rule)
            .map(|l| Ok(l.map(Cow::Owned)?))
            .collect::<Result<Vec<_>>>()?;
        output.insert(0, Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_START));
        output.push(Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_END));
        lines.splice(range_of_rule_providers, output);
        let content = lines.join("\n");
        tokio::fs::write(rules_path, &content).await?;
        Ok(())
    }

    async fn update_surge_sub_logs_url(
        &self,
        sub_logs_path: impl AsRef<Path>,
        sub_logs_link: impl AsRef<str>,
    ) -> Result<()> {
        let content = tokio::fs::read_to_string(&sub_logs_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(format!(r#"const sub_logs_link = "{}""#, sub_logs_link.as_ref()));
        let content = lines.join("\n");
        tokio::fs::write(sub_logs_path, &content).await?;
        Ok(())
    }

    async fn update_conf(config_path: impl AsRef<Path>, header: SurgeHeader) -> Result<()> {
        let mut content = tokio::fs::read_to_string(&config_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(header.to_string());
        content = lines.join("\n");
        tokio::fs::write(&config_path, &content).await?;
        Ok(())
    }
}

impl ClashConfig {
    async fn update_clash_config(
        &self,
        url: &UrlBuilder,
        raw_profile: ClashProfile,
        secret: impl AsRef<str>,
    ) -> Result<()> {
        let mut template = ClashProfile::template()?;
        template.patch(raw_profile)?;
        template.convert(url)?;
        template.secret = Some(secret.as_ref().to_string());
        let clash_config = ClashRenderer::render_profile(&template)?;
        let main_sub_path = self.main_sub_path();
        if !main_sub_path.is_file() {
            if let Some(parent) = main_sub_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(main_sub_path, clash_config).await?;
        Ok(())
    }
}

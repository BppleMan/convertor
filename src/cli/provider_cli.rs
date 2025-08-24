use crate::common::config::ConvertorConfig;
use crate::common::config::provider_config::Provider;
use crate::common::config::proxy_client_config::{ClashConfig, ProxyClient, ProxyClientConfig, SurgeConfig};
use crate::core::convertor_url::{ConvertorUrl, ConvertorUrlType};
use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::extract_policies_for_rule_provider;
use crate::core::profile::policy::Policy;
use crate::core::profile::rule::Rule;
use crate::core::profile::surge_header::SurgeHeader;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::renderer::surge_renderer::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START};
use crate::core::result::ParseResult;
use crate::core::url_builder::{HostPort, UrlBuilder, UrlBuilderError};
use crate::provider_api::ProviderApi;
use crate::server::query::ConvertorQuery;
use axum_extra::headers::UserAgent;
use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::path::Path;
use url::Url;

#[derive(Default, Debug, Clone, Args)]
pub struct ProviderCmd {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: ProxyClient,

    /// 订阅提供商
    #[arg(value_enum, default_value_t = Provider::BosLife)]
    pub provider: Provider,

    /// 订阅链接, 可以是 转换器链接(profile_url) 或 原始订阅链接(raw_url)
    /// 将会自动判断链接类型, 仅根据 host 部分与 server 进行匹配
    /// 可选的参数, 如果不指定, 则通过 provider 来获取 raw_url
    #[arg()]
    pub url: Option<Url>,

    /// convertor 所在服务器的地址
    /// 格式为 `http://host:port`
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

    /// 是否重置订阅链接
    #[arg(short, long, default_value_t = false)]
    pub reset: bool,
}

pub struct ProviderCli {
    pub config: ConvertorConfig,
    pub api_map: HashMap<Provider, ProviderApi>,
}

#[allow(clippy::large_enum_variant)]
enum ClientProfile {
    Surge,
    Clash(ClashProfile),
}

impl ProviderCli {
    pub fn new(config: ConvertorConfig, api_map: HashMap<Provider, ProviderApi>) -> Self {
        Self { config, api_map }
    }

    pub async fn execute(&mut self, cmd: ProviderCmd) -> Result<(UrlBuilder, ProviderCliResult)> {
        let client = cmd.client;
        let provider = cmd.provider;
        let url_builder = self.create_url_builder(&cmd).await?;
        let api = self
            .api_map
            .get_mut(&provider)
            .ok_or(eyre!("无法取得订阅供应商的 api 实现: {}", &provider))?;
        api.set_sub_url(url_builder.sub_url.clone());
        let raw_profile_content = api
            .get_raw_profile(client, UserAgent::from_static("Surge Mac/8310"))
            .await?;
        let sub_host = url_builder.sub_url.host_port()?;
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
                let policies = extract_policies_for_rule_provider(&raw_profile.rules, sub_host);
                (ClientProfile::Clash(raw_profile), policies)
            }
        };

        let raw_url = url_builder.build_raw_url();
        let profile_url = url_builder.build_profile_url()?;
        let raw_profile_url = url_builder.build_raw_profile_url()?;
        let sub_logs_url = url_builder.build_sub_logs_url()?;
        let rule_provider_urls = policies
            .iter()
            .map(|policy| url_builder.build_rule_provider_url(policy))
            .collect::<Result<Vec<_>, UrlBuilderError>>()?;
        let result = ProviderCliResult {
            raw_url,
            raw_profile_url,
            profile_url,
            sub_logs_url,
            rule_provider_urls,
        };

        // 副作用逻辑后置，主流程只负责数据流
        if cmd.update {
            match client_profile {
                ClientProfile::Surge => {
                    self.update_surge_config(&url_builder, &policies).await?;
                }
                ClientProfile::Clash(profile) => {
                    self.update_clash_config(&url_builder, profile).await?;
                }
            }
        }
        Ok((url_builder, result))
    }

    pub fn post_execute(&self, _url_builder: UrlBuilder, result: ProviderCliResult) {
        println!("{result}");
    }

    async fn create_url_builder(&self, cmd: &ProviderCmd) -> Result<UrlBuilder> {
        let ProviderCmd {
            client,
            provider,
            url,
            server,
            interval,
            strict,
            reset,
            update: _,
        } = cmd;

        // 从 client_config 中取参数的 fallback
        let client_config = self
            .config
            .clients
            .get(client)
            .ok_or(eyre!("无法取得代理客户端的配置: {}", client))?;

        let server = server.clone().unwrap_or_else(|| self.config.server.clone());
        let mut enc_secret = None;
        let mut enc_sub_url = None;
        let mut interval = interval
            .as_ref()
            .map(|i| *i)
            .unwrap_or_else(|| client_config.interval());
        let mut strict = strict.as_ref().map(|s| *s).unwrap_or_else(|| client_config.strict());

        let url_type = self.detect_url(cmd);
        let sub_url = match (url_type, reset) {
            // Get sub_url
            (None, false) => {
                self.api_map
                    .get(&provider)
                    .ok_or(eyre!("无法取得订阅供应商的 api 实现: {}", &provider))?
                    .get_sub_url()
                    .await?
            }
            // Reset sub_url
            (None, true) => {
                self.api_map
                    .get(&provider)
                    .ok_or(eyre!("无法取得订阅供应商的 api 实现: {}", &provider))?
                    .reset_sub_url()
                    .await?
            }
            // Use sub_url
            (Some(ConvertorUrlType::Raw), _) => url.clone().unwrap(),
            // Decode profile_url
            (Some(ConvertorUrlType::Profile), _) => {
                let profile_query = url.as_ref().and_then(Url::query).ok_or(eyre!("订阅链接缺少查询参数"))?;
                let query = ConvertorQuery::parse_from_query_string(
                    profile_query,
                    &self.config.secret,
                    server.clone(),
                    *client,
                    *provider,
                )?;
                enc_secret = query.enc_secret.clone();
                enc_sub_url = Some(query.enc_sub_url.clone());
                interval = query.interval;
                strict = query.strict.unwrap_or(strict);
                query.sub_url.clone()
            }
            _ => unreachable!("不支持的订阅链接类型"),
        };

        let url_builder = UrlBuilder::new(
            self.config.secret.clone(),
            enc_secret,
            *client,
            *provider,
            server,
            sub_url,
            enc_sub_url,
            interval,
            strict,
        )?;

        Ok(url_builder)
    }

    fn detect_url(&self, cmd: &ProviderCmd) -> Option<ConvertorUrlType> {
        let ProviderCmd { url, server, .. } = cmd;
        let server = server
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.config.server.to_string());
        url.as_ref().map(|url| {
            if url.as_str().starts_with(&server) {
                ConvertorUrlType::Profile
            } else {
                ConvertorUrlType::Raw
            }
        })
    }

    async fn update_surge_config(
        &self,
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &Policy>,
    ) -> Result<()> {
        if let Some(ProxyClientConfig::Surge(config)) = self.config.clients.get(&ProxyClient::Surge) {
            config.update_surge_config(url_builder, policies).await?;
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
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &Policy>,
    ) -> Result<()> {
        // 更新主订阅配置，即由 convertor 生成的订阅配置
        Self::update_conf(
            &self.main_profile_path(),
            url_builder.build_surge_header(ConvertorUrlType::Profile)?,
        )
        .await?;

        // 更新转发原始订阅配置，即由 convertor 生成的原始订阅配置
        if let Some(raw_profile_path) = self.raw_profile_path() {
            Self::update_conf(
                raw_profile_path,
                url_builder.build_surge_header(ConvertorUrlType::RawProfile)?,
            )
            .await?;
        }

        // 更新原始订阅配置，即由订阅提供商生成的订阅配置，如果存在的话
        if let Some(raw_sub_path) = self.raw_sub_path() {
            Self::update_conf(raw_sub_path, url_builder.build_surge_header(ConvertorUrlType::Raw)?).await?;
        }

        // 更新 rules.dconf 中的 RULE-SET 规则，规则提供者将从 policies 中生成 URL
        if let Some(rules_path) = self.rules_path() {
            self.update_surge_rule_providers(rules_path, url_builder, policies)
                .await?;
        }

        // 更新 subscription_logs.js 中的请求订阅日志的 URL
        if let Some(sub_logs_path) = self.sub_logs_path() {
            self.update_surge_sub_logs_url(sub_logs_path, url_builder.build_sub_logs_url()?)
                .await?;
        }
        Ok(())
    }

    async fn update_surge_rule_providers(
        &self,
        rules_path: impl AsRef<Path>,
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &Policy>,
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
            .into_iter()
            .map(|policy| {
                let name = SurgeRenderer::render_provider_name_for_policy(policy)?;
                let url = url_builder.build_rule_provider_url(policy)?;
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
        sub_logs_url: ConvertorUrl,
    ) -> Result<()> {
        let content = tokio::fs::read_to_string(&sub_logs_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(format!(r#"const sub_logs_link = "{}""#, sub_logs_url));
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
        url_builder: &UrlBuilder,
        raw_profile: ClashProfile,
        secret: impl AsRef<str>,
    ) -> Result<()> {
        let mut template = ClashProfile::template()?;
        template.patch(raw_profile)?;
        template.convert(url_builder)?;
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

#[derive(Debug, Clone)]
pub struct ProviderCliResult {
    pub raw_url: ConvertorUrl,
    pub raw_profile_url: ConvertorUrl,
    pub profile_url: ConvertorUrl,
    pub sub_logs_url: ConvertorUrl,
    pub rule_provider_urls: Vec<ConvertorUrl>,
}

impl Display for ProviderCliResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.raw_url)?;
        writeln!(f, "{}", self.profile_url)?;
        writeln!(f, "{}", self.raw_profile_url)?;
        writeln!(f, "{}", self.sub_logs_url)?;
        for link in &self.rule_provider_urls {
            writeln!(f, "{link}")?;
        }
        Ok(())
    }
}

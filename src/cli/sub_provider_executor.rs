use crate::api::SubProviderApi;
use crate::common::config::ConvertorConfig;
use crate::common::config::proxy_client::{ClashConfig, ProxyClient, SurgeConfig};
use crate::common::url::ConvertorUrl;
use crate::core::profile::Profile;
use crate::core::profile::clash_profile::ClashProfile;
use crate::core::profile::extract_policies_for_rule_provider;
use crate::core::profile::policy::Policy;
use crate::core::profile::rule::Rule;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::core::renderer::surge_renderer::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START};
use crate::core::result::ParseResult;
use crate::server::query::SubLogQuery;
use clap::{Args, Subcommand};
use color_eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use std::borrow::Cow;
use std::path::Path;
use url::Url;

#[derive(Default, Debug, Clone, Args)]
pub struct SubProviderCmd {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: ProxyClient,

    /// convertor 所在服务器的地址
    /// 格式为 `http://ip:port`
    #[arg(short, long)]
    pub server: Option<Url>,

    /// 订阅更新的间隔时间，单位为秒
    /// 默认为 86400 秒（24 小时）
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
    pub url_source: Option<ConvertorUrlSource>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConvertorUrlSource {
    /// 使用 订阅提供商API 获取最新订阅链接
    Get,

    /// 使用重置的原始订阅链接
    Reset,

    /// 解码 订阅提供商 的原始订阅链接
    Raw { raw_sub_url: Url },

    /// 解码 convertor 的完整订阅链接
    Decode { convertor_url: Url },
}

pub struct SubProviderExecutor {
    pub config: ConvertorConfig,
    pub api: SubProviderApi,
}

#[allow(clippy::large_enum_variant)]
enum ClientProfile {
    Surge,
    Clash(ClashProfile),
}

impl SubProviderExecutor {
    pub fn new(config: ConvertorConfig, api: SubProviderApi) -> Self {
        Self { config, api }
    }

    pub async fn execute(&self, cmd: SubProviderCmd) -> Result<()> {
        let client = cmd.client;
        let convertor_url = self.generate_convertor_url(&cmd).await?;
        let raw_profile_content = self.api.get_raw_profile(client).await?;
        let uni_sub_host = convertor_url.uni_sub_host()?;
        let (client_profile, policies) = match client {
            ProxyClient::Surge => {
                let raw_profile = SurgeProfile::parse(raw_profile_content)?;
                let polices = extract_policies_for_rule_provider(&raw_profile.rules, uni_sub_host);
                (ClientProfile::Surge, polices)
            }
            ProxyClient::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let polices = extract_policies_for_rule_provider(&raw_profile.rules, uni_sub_host);
                (ClientProfile::Clash(raw_profile), polices)
            }
        };

        let sub_log_query = SubLogQuery::new(&self.config.secret, 1, 20);
        let raw_sub_url = convertor_url.build_raw_sub_url()?;
        let sub_url = convertor_url.build_sub_url()?;
        let sub_logs_url = convertor_url.build_sub_logs_url(sub_log_query.encode_to_query_string()?)?;

        if cmd.update {
            match client_profile {
                ClientProfile::Surge => {
                    self.update_surge_config(&convertor_url, &sub_logs_url, &policies)
                        .await?;
                }
                ClientProfile::Clash(profile) => {
                    self.update_clash_config(&convertor_url, profile).await?;
                }
            }
        }

        println!("{}", "Raw Subscription url:".to_string().green().bold());
        println!("{}", raw_sub_url);
        println!("{}", "Convertor url:".to_string().green().bold());
        println!("{}", sub_url);
        println!("{}", "Subscription logs url:".to_string().green().bold());
        println!("{}", sub_logs_url);
        for policy in policies {
            match client {
                ProxyClient::Surge => println!(
                    "{}",
                    SurgeRenderer::render_provider_name_for_policy(&policy)?.green().bold()
                ),
                ProxyClient::Clash => println!(
                    "{}",
                    ClashRenderer::render_provider_name_for_policy(&policy)?.green().bold()
                ),
            }
            println!("{}", convertor_url.build_rule_provider_url(&policy)?)
        }
        Ok(())
    }

    async fn generate_convertor_url(&self, cmd: &SubProviderCmd) -> Result<ConvertorUrl> {
        let SubProviderCmd {
            client,
            server,
            interval,
            update: _update,
            strict,
            url_source,
        } = cmd;
        let secret = self.config.secret.clone();
        let server = server.clone().unwrap_or_else(|| self.config.server.clone());
        let interval = interval.unwrap_or_else(|| self.config.interval);
        let strict = strict.unwrap_or_else(|| self.config.strict);

        let convertor_url = match url_source {
            None => {
                let raw_sub_url = &self.config.provider.uni_sub_url;
                ConvertorUrl::new(secret, *client, server, raw_sub_url.clone(), interval, strict, None)?
            }
            Some(ConvertorUrlSource::Get) => {
                let raw_sub_url = self.api.get_raw_sub_url().await?;
                ConvertorUrl::new(secret, *client, server, raw_sub_url, interval, strict, None)?
            }
            Some(ConvertorUrlSource::Reset) => {
                let raw_sub_url = self.api.reset_raw_sub_url().await?;
                ConvertorUrl::new(secret, *client, server, raw_sub_url, interval, strict, None)?
            }
            Some(ConvertorUrlSource::Raw { raw_sub_url }) => {
                ConvertorUrl::new(secret, *client, server, raw_sub_url.clone(), interval, strict, None)?
            }
            Some(ConvertorUrlSource::Decode { convertor_url }) => {
                ConvertorUrl::parse_from_url(convertor_url, &self.config.secret)?
            }
        };
        Ok(convertor_url)
    }

    async fn update_surge_config(&self, url: &ConvertorUrl, sub_logs_url: &Url, policies: &[Policy]) -> Result<()> {
        if let Some(surge_config) = &self.config.client.surge {
            surge_config.update_surge_config(url, sub_logs_url, policies).await?;
        } else {
            eprintln!("{}", "Surge 配置未找到，请检查配置文件是否正确设置".red().bold());
        }
        Ok(())
    }

    async fn update_clash_config(&self, url: &ConvertorUrl, raw_profile: ClashProfile) -> Result<()> {
        if let Some(clash_config) = &self.config.client.clash {
            clash_config
                .update_clash_config(url, raw_profile, &self.config.secret)
                .await?;
        } else {
            eprintln!("{}", "Clash 配置未找到，请检查配置文件是否正确设置".red().bold());
        }
        Ok(())
    }
}

impl SurgeConfig {
    async fn update_surge_config(&self, url: &ConvertorUrl, sub_logs_url: &Url, policies: &[Policy]) -> Result<()> {
        // 更新主订阅配置，即由 convertor 生成的订阅配置
        let header = url.build_managed_config_header(false)?;
        Self::update_conf(&self.main_sub_path(), header).await?;

        // 更新原始订阅配置，即由订阅提供商生成的订阅配置，如果存在的话
        if let Some(raw_sub_path) = self.raw_sub_path() {
            let header = url.build_managed_config_header(true)?;
            Self::update_conf(raw_sub_path, header).await?;
        }

        // 更新 rules.dconf 中的 RULE-SET 规则，规则提供者将从 policies 中生成 URL
        if let Some(rules_path) = self.rules_path() {
            self.update_surge_rule_providers(rules_path, url, policies).await?;
        }

        // 更新 subscription_logs.js 中的请求订阅日志的 URL
        if let Some(sub_logs_path) = self.sub_logs_path() {
            self.update_surge_sub_logs_url(sub_logs_path, sub_logs_url.as_str())
                .await?;
        }
        Ok(())
    }

    async fn update_surge_rule_providers(
        &self,
        rules_path: impl AsRef<Path>,
        url: &ConvertorUrl,
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
                let url = url.build_rule_provider_url(policy)?;
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
        sub_logs_url: impl AsRef<str>,
    ) -> Result<()> {
        let content = tokio::fs::read_to_string(&sub_logs_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(format!(r#"const sub_logs_url = "{}""#, sub_logs_url.as_ref()));
        let content = lines.join("\n");
        tokio::fs::write(sub_logs_path, &content).await?;
        Ok(())
    }

    async fn update_conf(config_path: impl AsRef<Path>, header: impl AsRef<str>) -> Result<()> {
        let mut content = tokio::fs::read_to_string(&config_path).await?;
        let mut lines = content.lines().collect::<Vec<_>>();
        lines[0] = header.as_ref();
        content = lines.join("\n");
        tokio::fs::write(&config_path, &content).await?;
        Ok(())
    }
}

impl ClashConfig {
    async fn update_clash_config(
        &self,
        url: &ConvertorUrl,
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

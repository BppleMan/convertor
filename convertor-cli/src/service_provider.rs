use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use color_eyre::owo_colors::OwoColorize;
use convertor_core::api::ServiceApi;
use convertor_core::config::ConvertorConfig;
use convertor_core::core::profile::Profile;
use convertor_core::core::profile::clash_profile::ClashProfile;
use convertor_core::core::profile::extract_policies_for_rule_provider;
use convertor_core::core::profile::policy::Policy;
use convertor_core::core::profile::rule::Rule;
use convertor_core::core::profile::surge_profile::SurgeProfile;
use convertor_core::core::renderer::Renderer;
use convertor_core::core::renderer::clash_renderer::ClashRenderer;
use convertor_core::core::renderer::surge_renderer::SurgeRenderer;
use convertor_core::core::renderer::surge_renderer::{
    SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START,
};
use convertor_core::core::result::ParseResult;
use convertor_core::proxy_client::ProxyClient;
use convertor_core::url::{ConvertorUrl, Url};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
pub struct ServiceProviderArgs {
    /// 构造适用于不同客户端的订阅地址: [surge, clash]
    #[arg(value_enum)]
    pub client: ProxyClient,

    /// 是否重置订阅地址
    #[arg(short, long)]
    pub reset: bool,

    /// 是否更新本地配置文件
    #[arg(short, long)]
    pub update: bool,

    /// 服务商的原始订阅链接
    #[arg(long = "raw")]
    pub raw_sub_url: Option<Url>,

    /// 现有的已经转换订阅链接
    #[arg(long = "full")]
    pub convertor_url: Option<Url>,

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
    #[arg(long)]
    pub strict: Option<bool>,
}

pub struct SubscriptionService {
    pub config: ConvertorConfig,
    pub api: ServiceApi,
}

impl SubscriptionService {
    pub async fn execute(&mut self, args: ServiceProviderArgs) -> Result<()> {
        let client = args.client;
        let convertor_url = self.generate_url_builder(&args).await?;
        let raw_profile_content = self.api.get_raw_profile(client).await?;
        let policies = match client {
            ProxyClient::Surge => {
                let raw_profile = SurgeProfile::parse(raw_profile_content)?;
                let polices = extract_policies_for_rule_provider(&raw_profile.rules, convertor_url.raw_sub_host()?);
                if args.update {
                    self.update_surge_config(&convertor_url, &polices).await?;
                }
                polices
            }
            ProxyClient::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let polices = extract_policies_for_rule_provider(&raw_profile.rules, convertor_url.raw_sub_host()?);
                if args.update {
                    self.update_clash_config(&convertor_url, raw_profile).await?;
                }
                polices
            }
        };
        if matches!(client, ProxyClient::Clash) && args.update {
            return Ok(());
        }
        println!("{}", "Raw Subscription url:".to_string().green().bold());
        println!("{}", convertor_url.build_raw_sub_url()?);
        println!("{}", "Convertor url:".to_string().green().bold());
        println!("{}", convertor_url.build_sub_url()?);
        println!("{}", "Subscription logs url:".to_string().green().bold());
        println!("{}", convertor_url.build_sub_logs_url(&self.config.secret)?);
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

    async fn generate_url_builder(&self, args: &ServiceProviderArgs) -> Result<ConvertorUrl> {
        let ServiceProviderArgs {
            client,
            reset,
            update: _update,
            raw_sub_url,
            convertor_url,
            server,
            interval,
            strict,
        } = args;
        let secret = self.config.secret.clone();
        let server = server.clone().unwrap_or_else(|| self.config.server.clone());
        let interval = interval.unwrap_or_else(|| self.config.interval);
        let strict = strict.unwrap_or_else(|| self.config.strict);

        let convertor_url = if let Some(convertor_url) = convertor_url {
            ConvertorUrl::parse_from_url(convertor_url, &self.config.secret)?
        } else if *reset {
            let raw_sub_url = self.api.reset_raw_sub_url().await?;
            ConvertorUrl::new(secret, *client, server, raw_sub_url, interval, strict, None)?
        } else {
            let raw_sub_url = raw_sub_url
                .clone()
                .unwrap_or_else(|| self.config.service_config.raw_sub_url.clone());
            ConvertorUrl::new(secret, *client, server, raw_sub_url, interval, strict, None)?
        };
        Ok(convertor_url)
    }

    async fn update_surge_config(&self, url: &ConvertorUrl, policies: &[Policy]) -> Result<()> {
        let surge_config = SurgeConfig::try_new()?;
        surge_config.update_surge_config(url).await?;
        surge_config
            .update_surge_sub_logs_url(url.build_sub_logs_url(&self.config.secret)?)
            .await?;
        surge_config.update_surge_rule_providers(url, policies).await?;
        Ok(())
    }

    async fn update_clash_config(&self, url: &ConvertorUrl, raw_profile: ClashProfile) -> Result<()> {
        let clash_config = ClashConfig::try_new()?;
        clash_config
            .update_clash_config(url, raw_profile, &self.config.secret)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct SurgeConfig {
    #[allow(unused)]
    pub surge_dir: PathBuf,
    pub main_config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub rules_config_path: PathBuf,
    pub sub_logs_path: PathBuf,
}

impl SurgeConfig {
    pub fn try_new() -> Result<Self> {
        let icloud_env = std::env::var("ICLOUD")?;
        let icloud_path = Path::new(&icloud_env);
        let ns_surge_path = icloud_path
            .parent()
            .ok_or_eyre("not found icloud's parent")?
            .join("iCloud~com~nssurge~inc")
            .join("Documents");
        let main_config_path = ns_surge_path.join("surge").join("surge.conf");
        let default_config_path = ns_surge_path.join("surge").join("BosLife.conf");
        let rules_config_path = ns_surge_path.join("surge").join("rules.dconf");
        let sub_logs_path = ns_surge_path.join("surge").join("subscription_logs.js");
        Ok(Self {
            surge_dir: ns_surge_path,
            main_config_path,
            default_config_path,
            rules_config_path,
            sub_logs_path,
        })
    }

    pub async fn update_surge_config(&self, url: &ConvertorUrl) -> Result<()> {
        // update BosLife.conf subscription
        let header = url.build_managed_config_header(true)?;
        Self::update_conf(&self.default_config_path, header).await?;

        // update surge.conf subscription
        let header = url.build_managed_config_header(false)?;
        Self::update_conf(&self.main_config_path, header).await?;

        Ok(())
    }

    pub async fn update_surge_rule_providers(&self, url: &ConvertorUrl, policies: &[Policy]) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.rules_config_path).await?;
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
        tokio::fs::write(&self.rules_config_path, &content).await?;
        Ok(())
    }

    pub async fn update_surge_sub_logs_url(&self, sub_logs_url: impl AsRef<str>) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.sub_logs_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(format!(r#"const sub_logs_url = "{}""#, sub_logs_url.as_ref()));
        let content = lines.join("\n");
        tokio::fs::write(&self.sub_logs_path, &content).await?;
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

pub struct ClashConfig {
    pub clash_dir: PathBuf,
    pub main_config_path: PathBuf,
}

impl ClashConfig {
    pub fn try_new() -> Result<Self> {
        let home_env = std::env::var("HOME")?;
        let home_path = Path::new(&home_env);
        let clash_dir = home_path.join(".config").join("mihomo");
        let main_config_path = clash_dir.join("config.yaml");
        Ok(Self {
            clash_dir,
            main_config_path,
        })
    }

    pub async fn update_clash_config(
        &self,
        url: &ConvertorUrl,
        raw_profile: ClashProfile,
        secret: impl AsRef<str>,
    ) -> Result<()> {
        let mut template = ClashProfile::template()?;
        template.merge(raw_profile)?;
        template.optimize(url)?;
        template.secret = Some(secret.as_ref().to_string());
        let clash_config = ClashRenderer::render_profile(&template)?;
        if !self.main_config_path.is_file() {
            if let Some(parent) = self.main_config_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(&self.main_config_path, clash_config).await?;
        Ok(())
    }
}

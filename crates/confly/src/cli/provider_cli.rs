use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use convertor::common::config::ConvertorConfig;
use convertor::common::config::provider_config::Provider;
use convertor::common::config::proxy_client_config::ProxyClient;
use convertor::core::profile::Profile;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::extract_policies_for_rule_provider;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::provider_api::ProviderApi;
use convertor::url::convertor_url::{ConvertorUrl, ConvertorUrlType};
use convertor::url::query::ConvertorQuery;
use convertor::url::url_builder::{HostPort, UrlBuilder};
use convertor::url::url_error::UrlBuilderError;
use headers::UserAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use url::Url;

#[derive(Default, Debug, Clone, Hash, Args)]
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

    /// 是否重置订阅链接
    #[arg(short, long, default_value_t = false)]
    pub reset: bool,

    #[cfg(feature = "update")]
    /// 是否更新本地订阅文件
    #[arg(short, long, default_value_t = false)]
    pub update: bool,
}

pub struct ProviderCli {
    pub config: ConvertorConfig,
    pub api_map: HashMap<Provider, ProviderApi>,
}

#[allow(clippy::large_enum_variant)]
enum ClientProfile {
    Surge,
    #[cfg(feature = "update")]
    Clash(ClashProfile),
    #[cfg(not(feature = "update"))]
    Clash,
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
        let (_client_profile, policies) = match client {
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
                #[cfg(feature = "update")]
                {
                    (ClientProfile::Clash(raw_profile), policies)
                }
                #[cfg(not(feature = "update"))]
                {
                    (ClientProfile::Clash, policies)
                }
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

        #[cfg(feature = "update")]
        // 副作用逻辑后置，主流程只负责数据流
        if cmd.update {
            match _client_profile {
                ClientProfile::Surge => {
                    super::update::update_surge_config(&self.config, &url_builder, &policies).await?;
                }
                ClientProfile::Clash(profile) => {
                    super::update::update_clash_config(&self.config, &url_builder, profile).await?;
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
            #[cfg(feature = "update")]
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
                    .get(provider)
                    .ok_or(eyre!("无法取得订阅供应商的 api 实现: {}", &provider))?
                    .get_sub_url()
                    .await?
            }
            // Reset sub_url
            (None, true) => {
                self.api_map
                    .get(provider)
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCliResult {
    pub raw_url: ConvertorUrl,
    pub raw_profile_url: ConvertorUrl,
    pub profile_url: ConvertorUrl,
    pub sub_logs_url: ConvertorUrl,
    pub rule_provider_urls: Vec<ConvertorUrl>,
}

impl Display for ProviderCliResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.raw_url.r#type.blue())?;
        writeln!(f, "{}", self.raw_url)?;
        writeln!(f, "{}", self.profile_url.r#type.blue())?;
        writeln!(f, "{}", self.profile_url)?;
        writeln!(f, "{}", self.raw_profile_url.r#type.blue())?;
        writeln!(f, "{}", self.raw_profile_url)?;
        writeln!(f, "{}", self.sub_logs_url.r#type.blue())?;
        writeln!(f, "{}", self.sub_logs_url)?;
        writeln!(f, "{}", ConvertorUrlType::RuleProvider.to_string().blue())?;
        for link in &self.rule_provider_urls {
            writeln!(f, "{link}")?;
        }
        Ok(())
    }
}

impl ProviderCmd {
    pub fn snapshot_name(&self) -> String {
        let client = self.client.to_string();
        let provider = self.provider.to_string();
        let url = self
            .url
            .as_ref()
            .map_or("no_url".to_string(), |url| url.host_port().unwrap());
        let server = self
            .server
            .as_ref()
            .map_or("no_server".to_string(), |server| server.to_string());
        let interval = self
            .interval
            .map_or("no_interval".to_string(), |interval| interval.to_string());
        let strict = self.strict.map_or("no_strict".to_string(), |_| "strict".to_string());
        let reset = if self.reset { "reset" } else { "no_reset" };

        #[cfg(feature = "update")]
        let update = if self.update { "update" } else { "no_update" };

        #[cfg(not(feature = "update"))]
        let update = "no_update";

        format!("{client}-{provider}-{url}-{server}-{interval}-{strict}-{reset}-{update}")
    }
}

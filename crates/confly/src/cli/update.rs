use crate::config::{ClientConfig, ConflyConfig};
use color_eyre::owo_colors::OwoColorize;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::Profile;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::rule::Rule;
use convertor::core::profile::surge_header::SurgeHeader;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::renderer::surge_renderer::{
    SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START, SurgeRenderer,
};
use convertor::url::convertor_url::{ConvertorUrl, ConvertorUrlType};
use convertor::url::url_builder::UrlBuilder;
use std::borrow::Cow;
use std::path::Path;
// pub async fn update_surge_config(
//     config: &ConflyConfig,
//     url_builder: &UrlBuilder,
//     policies: impl IntoIterator<Item = &Policy>,
// ) -> color_eyre::Result<()> {
//     if let Some(client_config) = config.clients.get(&ProxyClient::Surge) {
//         surge_config.update_surge_config(url_builder, policies).await?;
//     } else {
//         eprintln!("{}", "Surge 配置未找到，请检查配置文件是否正确设置".red().bold());
//     }
//     Ok(())
// }
//
// pub async fn update_clash_config(
//     config: &ConvertorConfig,
//     url_builder: &UrlBuilder,
//     raw_profile: ClashProfile,
// ) -> color_eyre::Result<()> {
//     if let Some(ProxyClientConfig::Clash(clash_config)) = config.clients.get(&ProxyClient::Clash) {
//         clash_config
//             .update_clash_config(url_builder, raw_profile, &config.secret)
//             .await?;
//     } else {
//         eprintln!("{}", "Clash 配置未找到，请检查配置文件是否正确设置".red().bold());
//     }
//     Ok(())
// }

impl ClientConfig {
    pub async fn update_surge_config(
        &self,
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &Policy>,
    ) -> color_eyre::Result<()> {
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

        Ok(())
    }

    async fn update_surge_rule_providers(
        &self,
        rules_path: impl AsRef<Path>,
        url_builder: &UrlBuilder,
        policies: impl IntoIterator<Item = &Policy>,
    ) -> color_eyre::Result<()> {
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
                let name = SurgeRenderer::render_provider_name_for_policy(policy);
                let url = url_builder.build_rule_provider_url(policy)?;
                Ok(Rule::surge_rule_provider(policy, name, url))
            })
            .collect::<color_eyre::Result<Vec<_>>>()?;
        let mut output = provider_rules
            .iter()
            .map(SurgeRenderer::render_rule)
            .map(|l| Ok(l.map(Cow::Owned)?))
            .collect::<color_eyre::Result<Vec<_>>>()?;
        output.insert(0, Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_START));
        output.push(Cow::Borrowed(SURGE_RULE_PROVIDER_COMMENT_END));
        lines.splice(range_of_rule_providers, output);
        let content = lines.join("\n");
        tokio::fs::write(rules_path, &content).await?;
        Ok(())
    }

    async fn update_conf(config_path: impl AsRef<Path>, header: SurgeHeader) -> color_eyre::Result<()> {
        let mut content = tokio::fs::read_to_string(&config_path).await?;
        let mut lines = content.lines().map(Cow::Borrowed).collect::<Vec<_>>();
        lines[0] = Cow::Owned(header.to_string());
        content = lines.join("\n");
        tokio::fs::write(&config_path, &content).await?;
        Ok(())
    }

    pub async fn update_clash_config(
        &self,
        url_builder: &UrlBuilder,
        raw_profile: ClashProfile,
        secret: impl AsRef<str>,
    ) -> color_eyre::Result<()> {
        let mut template = ClashProfile::template()?;
        template.patch(raw_profile)?;
        template.convert(url_builder)?;
        template.secret = Some(secret.as_ref().to_string());
        let clash_config = ClashRenderer::render_profile(&template)?;
        let main_sub_path = self.main_profile_path();
        if !main_sub_path.is_file() {
            if let Some(parent) = main_sub_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(main_sub_path, clash_config).await?;
        Ok(())
    }
}

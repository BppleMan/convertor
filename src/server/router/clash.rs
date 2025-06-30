use crate::profile::clash_profile::ClashProfile;
use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::server::router::AppState;
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashQuery {
    pub raw_url: String,
    #[serde(default)]
    pub policies: Option<String>,
    #[serde(default)]
    pub boslife: Option<bool>,
}

pub(super) async fn profile_impl(state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
    let mut template = ClashProfile::template()?;
    template.merge(raw_profile, &url_builder, &state.convertor_config.secret)?;
    Ok(template.serialize())
}

pub(super) async fn rule_set_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Option<RuleSetPolicy>,
) -> Result<String> {
    let raw_profile = ClashProfile::from_str(&raw_profile)?;
    let sub_host = url_builder
        .service_url
        .host_str()
        .ok_or(eyre!("错误的订阅 URL, 未能解析出 host"))?;
    policy
        .map(|policy| {
            raw_profile
                .generate_rule_provider(policy, sub_host)
                .ok_or_else(|| eyre!("未能找到匹配策略的规则以生成 Rule Provider"))
        })
        .ok_or_else(|| eyre!("未知的查询参数: 缺少 policies 或 boslife 标志"))?
}

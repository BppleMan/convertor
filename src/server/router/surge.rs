use crate::profile::rule_set_policy::RuleSetPolicy;
use crate::profile::surge_profile::SurgeProfile;
use crate::server::router::AppState;
use crate::subscription::url_builder::UrlBuilder;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::sync::Arc;

pub(super) async fn profile_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
) -> Result<String> {
    let mut profile = SurgeProfile::new(raw_profile);
    profile.optimize(url_builder)?;
    Ok(profile.to_string())
}

pub(super) async fn rule_set_impl(
    _state: Arc<AppState>,
    url_builder: UrlBuilder,
    raw_profile: String,
    policy: Option<RuleSetPolicy>,
) -> Result<String> {
    let raw_profile = SurgeProfile::new(raw_profile);
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

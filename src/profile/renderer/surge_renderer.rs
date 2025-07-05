use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::ProxyGroup;
use crate::profile::core::rule::Rule;
use crate::profile::renderer::Result;
use crate::profile::surge_profile::SurgeProfile;
use indexmap::IndexMap;
use std::fmt::Write;
use tracing::instrument;

pub struct SurgeRenderer;

impl SurgeRenderer {
    #[instrument(skip_all)]
    pub fn render_profile(profile: &SurgeProfile) -> Result<String> {
        let SurgeProfile {
            header,
            general,
            proxies,
            proxy_groups,
            rules,
            url_rewrite,
            misc,
        } = profile;

        let output = [
            Self::render_header(header)?,
            Self::render_general(general)?,
            Self::render_proxies(proxies)?,
            Self::render_proxy_groups(proxy_groups)?,
            Self::render_rules(rules)?,
            Self::render_url_rewrite(url_rewrite)?,
            Self::render_misc(misc)?,
        ]
        .join("\n");

        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_header(header: &str) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "{}", header)?;
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_general(general: &[String]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "[General]")?;
        for line in general {
            writeln!(&mut output, "{}", line)?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_proxies(proxies: &[Proxy]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "[Proxy]")?;
        for proxy in proxies {
            if let Some(comment) = &proxy.comment {
                writeln!(&mut output, "{}", comment)?;
            }
            write!(
                &mut output,
                "{}={},{},{},{}",
                proxy.name, proxy.r#type, proxy.server, proxy.port, proxy.password
            )?;
            if let Some(cipher) = &proxy.cipher {
                write!(&mut output, ",encrypt-method={}", cipher)?;
            }
            if let Some(udp) = proxy.udp {
                write!(&mut output, ",udp-replay={}", udp)?;
            }
            if let Some(tfo) = proxy.tfo {
                write!(&mut output, ",tfo={}", tfo)?;
            }
            if let Some(sni) = &proxy.sni {
                write!(&mut output, ",sni={}", sni)?;
            }
            if let Some(skip_cert_verify) = proxy.skip_cert_verify {
                write!(&mut output, ",skip-cert-verify={}", skip_cert_verify)?;
            }
            writeln!(&mut output)?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "[Proxy Group]")?;
        for group in proxy_groups {
            if let Some(comment) = &group.comment {
                writeln!(&mut output, "{}", comment)?;
            }
            write!(&mut output, "{}={}", group.name, group.r#type.as_str())?;
            if !group.proxies.is_empty() {
                write!(&mut output, ",{}", group.proxies.join(","))?;
            }
            writeln!(&mut output)?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_rules(rules: &[Rule]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "[Rule]")?;
        for rule in rules {
            writeln!(&mut output, "{}", Self::render_rule(rule)?)?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    pub fn render_rule(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        if let Some(comment) = &rule.comment {
            writeln!(&mut output, "{}", comment)?;
        }
        write!(&mut output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = &rule.value {
            write!(&mut output, ",{}", value)?;
        }
        write!(&mut output, "{}", Self::render_policy(&rule.policy)?)?;
        Ok(output)
    }

    pub fn render_policy(policy: &Policy) -> Result<String> {
        let mut output = String::new();
        write!(&mut output, ",{}", policy.name)?;
        if let Some(option) = &policy.option {
            write!(&mut output, ",{}", option)?;
        }
        Ok(output)
    }

    // 渲染规则集中的单个规则，这是不需要带 policy 的
    #[instrument(skip_all)]
    pub fn render_rule_set(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        write!(
            &mut output,
            "{},{}",
            rule.rule_type.as_str(),
            rule.value.as_ref().expect("规则集中的规则必须有 value")
        )?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_url_rewrite(url_rewrite: &[String]) -> Result<String> {
        let mut output = String::new();
        writeln!(&mut output, "[URL Rewrite]")?;
        for line in url_rewrite {
            writeln!(&mut output, "{}", line)?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_misc(misc: &IndexMap<String, Vec<String>>) -> Result<String> {
        let mut output = String::new();
        for (key, values) in misc {
            writeln!(&mut output, "{}", key)?;
            for value in values {
                writeln!(&mut output, "{}", value)?;
            }
            writeln!(&mut output)?;
        }
        writeln!(&mut output)?;
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn render_policy_for_comment(policy: &Policy) -> String {
        let mut output = String::new();
        write!(output, "[").expect("无法写入 Surge 注释");
        if policy.is_subscription {
            write!(output, "Subscription").expect("无法写入 Surge 注释");
        } else {
            write!(output, "{}", policy.name).expect("无法写入 Surge 注释");
        }
        if let Some(option) = policy.option.as_ref() {
            write!(output, ": {}", option).expect("无法写入 Surge 注释");
        }
        write!(output, "]").expect("无法写入 Surge 注释");
        output
    }
}

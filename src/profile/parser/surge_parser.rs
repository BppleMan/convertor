use crate::profile::core::policy::Policy;
use crate::profile::core::proxy::Proxy;
use crate::profile::core::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::profile::core::rule::{Rule, RuleType};
use crate::profile::surge_profile::SurgeProfile;
use color_eyre::eyre::eyre;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::Write;
use tracing::instrument;

#[allow(unused)]
pub const SECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\[[^\[\]]+]$"#).unwrap());
#[allow(unused)]
pub const COMMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(;|#|//)").unwrap());
#[allow(unused)]
pub const INLINE_COMMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(;|#|//).*$").unwrap());

pub const MANAGED_CONFIG_HEADER: &str = "MANAGED-CONFIG";
pub const GENERAL_SECTION: &str = "[General]";
pub const PROXY_SECTION: &str = "[Proxy]";
pub const PROXY_GROUP_SECTION: &str = "[Proxy Group]";
pub const RULE_SECTION: &str = "[Rule]";
pub const URL_REWRITE_SECTION: &str = "[URL Rewrite]";

pub struct SurgeParser;

impl SurgeParser {
    #[instrument(skip_all)]
    pub fn parse_profile(content: String) -> color_eyre::Result<SurgeProfile> {
        let mut sections = Self::parse_raw(&content);
        let header = Self::parse_header(&mut sections)?;
        let general = Self::parse_general(&mut sections)?;
        let proxies = Self::parse_proxies(&mut sections)?;
        let proxy_groups = Self::parse_proxy_groups(&mut sections)?;
        let rules = Self::parse_rules(&mut sections)?;
        let url_rewrite = Self::parse_url_rewrite(&mut sections)?;
        let misc = sections
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.into_iter().map(str::to_owned).collect()))
            .collect();
        Ok(SurgeProfile {
            header,
            general,
            proxies,
            proxy_groups,
            rules,
            url_rewrite,
            misc,
        })
    }

    // #[instrument(skip_all)]
    // pub fn parse_raw(content: String) -> IndexMap<String, Vec<String>> {
    //     let mut contents = content.lines().map(|s| s.to_string()).collect::<VecDeque<String>>();
    //     let mut sections = IndexMap::new();
    //
    //     let mut section = MANAGED_CONFIG_HEADER.to_string();
    //     let mut cursor = 0;
    //     while cursor < contents.len() {
    //         let line = &contents[cursor];
    //         if line.starts_with('[') && line.ends_with(']') {
    //             let new_section = line.to_string();
    //             // 跳过 [Section] 所在行
    //             contents.pop_front();
    //             cursor -= 1;
    //             sections.insert(
    //                 std::mem::replace(&mut section, new_section),
    //                 contents.drain(0..cursor).collect::<Vec<_>>(),
    //             );
    //             cursor = 0;
    //         }
    //         cursor += 1;
    //     }
    //     sections.insert(section, contents.into_iter().collect());
    //
    //     sections
    // }

    #[instrument(skip_all)]
    pub fn parse_raw(content: &str) -> IndexMap<&str, Vec<&str>> {
        let mut sections = IndexMap::new();
        let mut current_section = MANAGED_CONFIG_HEADER;
        let mut current_lines = Vec::new();

        for line in content.lines() {
            if line.starts_with('[') && line.ends_with(']') {
                sections.insert(current_section, std::mem::take(&mut current_lines));
                current_section = line;
            } else {
                current_lines.push(line);
            }
        }

        sections.insert(current_section, current_lines);
        sections
    }

    #[instrument(skip_all)]
    pub fn parse_header(sections: &mut IndexMap<&str, Vec<&str>>) -> color_eyre::Result<String> {
        let header = sections
            .shift_remove(MANAGED_CONFIG_HEADER)
            .ok_or_else(|| eyre!("没有找到 MANAGED-CONFIG"))?
            .join("\n")
            .trim()
            .to_owned();
        Ok(header)
    }

    #[instrument(skip_all)]
    pub fn parse_general(sections: &mut IndexMap<&str, Vec<&str>>) -> color_eyre::Result<Vec<String>> {
        sections
            .shift_remove(GENERAL_SECTION)
            .map(|section| section.into_iter().map(|s| s.to_owned()).collect())
            .ok_or_else(|| eyre!("没有找到 [General]"))
    }

    #[instrument(skip_all)]
    pub fn parse_proxies(sections: &mut IndexMap<&str, Vec<&str>>) -> color_eyre::Result<Vec<Proxy>> {
        let proxies = sections
            .shift_remove(PROXY_SECTION)
            .ok_or_else(|| eyre!("没有找到 [Proxy]"))?;
        let proxies = Self::parse_comment(proxies, Self::parse_proxy, |proxy, comment| proxy.comment = comment)?;
        Ok(proxies)
    }

    // #[instrument(skip_all)]
    // pub fn parse_proxy(line: &str) -> color_eyre::Result<Proxy> {
    //     let line = Self::trim_line_comment(line);
    //     let Some((name, value)) = line.split_once('=') else {
    //         return Err(eyre!("Proxy 格式错误: {}", line));
    //     };
    //     let name = name.trim().to_string();
    //     let mut fields = value
    //         .split(',')
    //         .map(|field| field.trim().to_owned())
    //         .collect::<VecDeque<_>>();
    //     let Some(r#type) = fields.pop_front() else {
    //         return Err(eyre!("Proxy type 缺失: {}", line));
    //     };
    //     let Some(server) = fields.pop_front() else {
    //         return Err(eyre!("Proxy server 缺失: {}", line));
    //     };
    //     let Some(port) = fields.pop_front().and_then(|p| p.parse::<u16>().ok()) else {
    //         return Err(eyre!("Proxy port 缺失: {}", line));
    //     };
    //     let mut field_map = fields
    //         .into_iter()
    //         .filter_map(|f| {
    //             f.split_once('=')
    //                 .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
    //         })
    //         .collect::<HashMap<_, _>>();
    //     let Some(password) = field_map.remove("password") else {
    //         return Err(eyre!("Proxy password 缺失: {}", line));
    //     };
    //     let udp = field_map.remove("udp-replay").and_then(|u| u.parse::<bool>().ok());
    //     let tfo = field_map.remove("tfo").and_then(|t| t.parse::<bool>().ok());
    //     let cipher = field_map.remove("encrypt-method");
    //     let sni = field_map.remove("sni");
    //     let skip_cert_verify = field_map.get("skip-cert-verify").and_then(|s| s.parse::<bool>().ok());
    //     #[rustfmt::skip]
    //     let proxy = Proxy { name, r#type, server, port, password, udp, tfo, cipher, sni, skip_cert_verify, comment: None, };
    //     Ok(proxy)
    // }

    #[instrument(skip_all)]
    pub fn parse_proxy(line: &str) -> color_eyre::Result<Proxy> {
        let line = Self::trim_line_comment(line);

        let (name, value) = line.split_once('=').ok_or_else(|| eyre!("Proxy 格式错误: {line}"))?;

        let name = name.trim();

        let mut fields = value.split(',').map(str::trim);

        let r#type = fields.next().ok_or_else(|| eyre!("Proxy type 缺失: {line}"))?;
        let server = fields.next().ok_or_else(|| eyre!("Proxy server 缺失: {line}"))?;
        let port = fields
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .ok_or_else(|| eyre!("Proxy port 缺失或非法: {line}"))?;

        // 避免 HashMap，直接解构
        let mut password = None;
        let mut udp = None;
        let mut tfo = None;
        let mut cipher = None;
        let mut sni = None;
        let mut skip_cert_verify = None;

        for kv in fields {
            if let Some((k, v)) = kv.split_once('=') {
                let k = k.trim();
                let v = v.trim();
                match k {
                    "password" => password = Some(v),
                    "udp-replay" => udp = v.parse::<bool>().ok(),
                    "tfo" => tfo = v.parse::<bool>().ok(),
                    "encrypt-method" => cipher = Some(v),
                    "sni" => sni = Some(v),
                    "skip-cert-verify" => skip_cert_verify = v.parse::<bool>().ok(),
                    _ => {} // 忽略未知字段
                }
            }
        }

        let password = password.ok_or_else(|| eyre!("Proxy password 缺失: {line}"))?;

        Ok(Proxy {
            name: name.to_string(),
            r#type: r#type.to_string(),
            server: server.to_string(),
            port,
            password: password.to_string(),
            udp,
            tfo,
            cipher: cipher.map(str::to_string),
            sni: sni.map(str::to_string),
            skip_cert_verify,
            comment: None,
        })
    }

    #[instrument(skip_all)]
    pub fn parse_proxy_groups(sections: &mut IndexMap<&str, Vec<&str>>) -> color_eyre::Result<Vec<ProxyGroup>> {
        let proxy_groups = sections
            .shift_remove(PROXY_GROUP_SECTION)
            .ok_or_else(|| eyre!("没有找到 [Proxy Group]"))?;
        let proxy_groups = Self::parse_comment(proxy_groups, Self::parse_proxy_group, |group, comment| {
            group.comment = comment
        })?;
        Ok(proxy_groups)
    }

    #[instrument(skip_all)]
    pub fn parse_proxy_group(line: &str) -> color_eyre::Result<ProxyGroup> {
        let line = Self::trim_line_comment(line.as_ref());
        let Some((name, value)) = line.split_once('=') else {
            return Err(eyre!("Proxy Group 格式错误: {}", line));
        };
        let mut fields = value.split(',');
        let Some(r#type) = fields
            .next()
            .map(str::trim)
            .and_then(|t| t.parse::<ProxyGroupType>().ok())
        else {
            return Err(eyre!("Proxy Group type 缺失: {}", line));
        };
        let proxies = fields.map(str::trim).map(str::to_string).collect::<Vec<_>>();
        let proxy_group = ProxyGroup {
            name: name.trim().to_string(),
            r#type,
            proxies,
            comment: None,
        };
        Ok(proxy_group)
    }

    #[instrument(skip_all)]
    pub fn parse_rules(sections: &mut IndexMap<&str, Vec<&str>>) -> color_eyre::Result<Vec<Rule>> {
        let rules = sections
            .shift_remove(RULE_SECTION)
            .ok_or_else(|| eyre!("没有找到 [Rule]"))?;
        let rules = Self::parse_comment(rules, Self::parse_rule, |rule, comment| rule.comment = comment)?;
        Ok(rules)
    }

    #[instrument(skip_all)]
    pub fn parse_rule(line: &str) -> color_eyre::Result<Rule> {
        let line = Self::trim_line_comment(line.as_ref());
        let fields = line.split(',').collect::<Vec<_>>();
        if fields.len() < 2 {
            return Err(eyre!("规则类型缺失或格式错误: {}", line));
        }
        let Ok(rule_type) = fields[0].trim().parse::<RuleType>() else {
            return Err(eyre!("规则类型缺失或格式错误: {}", line));
        };
        let (value, policy) = if fields.len() == 2 {
            let policy = Policy {
                name: fields[1].trim().to_owned(),
                option: None,
                is_subscription: false,
            };
            (None, policy)
        } else {
            let value = fields[1].trim().to_string();
            let policy = Policy {
                name: fields[2].trim().to_owned(),
                option: fields.get(3).map(|o| o.to_string()),
                is_subscription: false,
            };
            (Some(value), policy)
        };
        let rule = Rule {
            rule_type,
            value,
            policy,
            comment: None,
        };
        Ok(rule)
    }

    #[instrument(skip_all)]
    pub fn parse_url_rewrite(sections: &mut IndexMap<&str, Vec<&str>>) -> color_eyre::Result<Vec<String>> {
        sections
            .shift_remove(URL_REWRITE_SECTION)
            .map(|section| section.into_iter().map(str::to_owned).collect())
            .ok_or_else(|| eyre!("没有找到 [URL Rewrite]"))
    }

    #[instrument(skip_all)]
    fn parse_comment<T, F, C>(contents: Vec<&str>, parse: F, set_comment: C) -> color_eyre::Result<Vec<T>>
    where
        F: Fn(&str) -> color_eyre::Result<T>,
        C: Fn(&mut T, Option<String>),
    {
        let mut comment: Option<String> = None;
        let items = contents
            .into_iter()
            .filter_map(|line| {
                let line = line.trim();
                let is_comment = line.starts_with('#') || line.starts_with(';') || line.starts_with("//");
                if line.is_empty() || is_comment {
                    if let Some(comment) = comment.as_mut() {
                        write!(comment, "\n{}", line).expect("解析时无法写入注释");
                    } else {
                        comment = Some(line.to_owned());
                    }
                } else if let Ok(mut item) = parse(&line) {
                    set_comment(&mut item, comment.take());
                    return Some(item);
                }
                None
            })
            .collect::<Vec<_>>();
        Ok(items)
    }

    #[instrument(skip_all)]
    fn trim_line_comment(line: &str) -> &str {
        line.split_once("//")
            .map_or(line, |(left, _)| left)
            .split_once(";")
            .map_or(line, |(left, _)| left)
            .split_once("#")
            .map_or(line, |(left, _)| left)
    }
}

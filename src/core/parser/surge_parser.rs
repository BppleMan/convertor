use crate::core::error::ParseError;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::core::profile::rule::{Rule, RuleType};
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::result::ParseResult;
use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;
use tracing::{instrument, trace};
// pub const SECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\[[^\[\]]+]$"#).unwrap());
// pub const COMMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(;|#|//)").unwrap());
// pub const INLINE_COMMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(;|#|//).*$").unwrap());

pub const MANAGED_CONFIG_HEADER: &str = "MANAGED-CONFIG";
pub const GENERAL_SECTION: &str = "[General]";
pub const PROXY_SECTION: &str = "[Proxy]";
pub const PROXY_GROUP_SECTION: &str = "[Proxy Group]";
pub const RULE_SECTION: &str = "[Rule]";
pub const URL_REWRITE_SECTION: &str = "[URL Rewrite]";

pub struct SurgeParser;

impl SurgeParser {
    #[instrument(skip_all)]
    pub fn parse_profile(content: String) -> ParseResult<SurgeProfile> {
        let mut sections = Self::parse_raw(&content);
        let header = if let Some(section) = sections.remove(MANAGED_CONFIG_HEADER) {
            Self::parse_header(section)?
        } else {
            return Err(ParseError::SectionMissing(MANAGED_CONFIG_HEADER));
        };
        let general = if let Some(section) = sections.remove(GENERAL_SECTION) {
            Self::parse_general(section)?
        } else {
            return Err(ParseError::SectionMissing(GENERAL_SECTION));
        };
        let proxies = if let Some(section) = sections.remove(PROXY_SECTION) {
            Self::parse_proxies(section)?
        } else {
            return Err(ParseError::SectionMissing(PROXY_SECTION));
        };
        let proxy_groups = if let Some(section) = sections.remove(PROXY_GROUP_SECTION) {
            Self::parse_proxy_groups(section)?
        } else {
            return Err(ParseError::SectionMissing(PROXY_GROUP_SECTION));
        };
        let rules = if let Some(section) = sections.remove(RULE_SECTION) {
            Self::parse_rules(section)?
        } else {
            return Err(ParseError::SectionMissing(RULE_SECTION));
        };
        let url_rewrite = if let Some(section) = sections.remove(URL_REWRITE_SECTION) {
            Self::parse_url_rewrite(section)?
        } else {
            vec![]
        };
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
            policy_of_rules: HashMap::new(),
        })
    }

    #[instrument(skip_all)]
    pub fn parse_raw(content: &str) -> HashMap<&str, Vec<&str>> {
        let mut sections = HashMap::new();
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
    pub fn parse_header(section: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<String> {
        let mut output = String::new();
        for line in section {
            writeln!(output, "{}", line.as_ref())?;
        }
        Ok(output)
    }

    #[instrument(skip_all)]
    pub fn parse_general(section: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<Vec<String>> {
        Ok(section.into_iter().map(|s| s.as_ref().to_owned()).collect())
    }

    #[instrument(skip_all)]
    pub fn parse_url_rewrite(section: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<Vec<String>> {
        Ok(section.into_iter().map(|s| s.as_ref().to_owned()).collect())
    }

    #[instrument(skip_all)]
    pub fn parse_proxies(section: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<Vec<Proxy>> {
        Self::parse_comment(section, Self::parse_proxy, Proxy::set_comment)
    }

    #[instrument(skip_all)]
    pub fn parse_proxy(line: &str) -> ParseResult<Proxy> {
        let line = Self::trim_line_comment(line);

        let (name, value) = line.split_once('=').ok_or_else(|| ParseError::Proxy {
            line: 0,
            reason: format!("Proxy 格式错误, 应该为`name=value`: {line}"),
        })?;

        let name = name.trim();

        let mut fields = value.split(',').map(str::trim);

        let r#type = fields.next().ok_or_else(|| ParseError::Proxy {
            line: 0,
            reason: format!("Proxy 缺失 type: {line}"),
        })?;
        let server = fields.next().ok_or_else(|| ParseError::Proxy {
            line: 0,
            reason: format!("Proxy 缺失 server: {line}"),
        })?;
        let port = fields
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .ok_or_else(|| ParseError::Proxy {
                line: 0,
                reason: format!("Proxy 缺失 port 或格式错误: {line}"),
            })?;

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

        let password = password.ok_or_else(|| ParseError::Proxy {
            line: 0,
            reason: format!("Proxy 缺失 password: {line}"),
        })?;

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
    pub fn parse_proxy_groups(section: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<Vec<ProxyGroup>> {
        Self::parse_comment(section, Self::parse_proxy_group, ProxyGroup::set_comment)
    }

    #[instrument(skip_all)]
    pub fn parse_proxy_group(line: &str) -> ParseResult<ProxyGroup> {
        let line = Self::trim_line_comment(line);
        let Some((name, value)) = line.split_once('=') else {
            return Err(ParseError::ProxyGroup {
                line: 0,
                reason: format!("Proxy Group 格式错误, 应该为`name=value`: {line}"),
            });
        };
        let mut fields = value.split(',');
        let Some(r#type) = fields
            .next()
            .map(str::trim)
            .and_then(|t| t.parse::<ProxyGroupType>().ok())
        else {
            return Err(ParseError::ProxyGroup {
                line: 0,
                reason: format!("Proxy Group 缺失 type 或格式错误: {line}"),
            });
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
    pub fn parse_rules(section: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<Vec<Rule>> {
        Self::parse_comment(section, Self::parse_rule, Rule::set_comment)
    }

    #[instrument(skip_all)]
    pub fn parse_rule(line: &str) -> ParseResult<Rule> {
        let line = Self::trim_line_comment(line);
        let fields = line.split(',').collect::<Vec<_>>();
        let (value, policy) = match fields.len() {
            0 | 1 => {
                return Err(ParseError::Rule {
                    line: 0,
                    reason: format!("规则格式错误, 应该为`type,value[,policy[,option]]`: {line}"),
                });
            }
            2 => {
                let policy = Policy {
                    name: fields[1].trim().to_owned(),
                    option: None,
                    is_subscription: false,
                };
                (None, policy)
            }
            _ => {
                let value = fields[1].trim().to_string();
                let policy = Policy {
                    name: fields[2].trim().to_owned(),
                    option: fields.get(3).map(|o| o.to_string()),
                    is_subscription: false,
                };
                (Some(value), policy)
            }
        };

        let rule_type = RuleType::from_str(fields[0].trim())?;

        let rule = Rule {
            rule_type,
            value,
            policy,
            comment: None,
        };
        Ok(rule)
    }

    #[instrument(skip_all)]
    fn parse_comment<R, F, C>(
        contents: impl IntoIterator<Item = impl AsRef<str>>,
        parse: F,
        set_comment: C,
    ) -> ParseResult<Vec<R>>
    where
        F: Fn(&str) -> ParseResult<R>,
        C: Fn(&mut R, Option<String>),
    {
        let mut items = vec![];
        let mut comment: Option<String> = None;
        for line in contents {
            let line = line.as_ref().trim();
            match line {
                line if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with("//") => {
                    match comment.as_mut() {
                        None => comment = Some(line.to_string()),
                        Some(comment) => writeln!(comment, "{}", line)?,
                    }
                }
                _ => match parse(line) {
                    Ok(mut item) => {
                        set_comment(&mut item, comment.take());
                        items.push(item)
                    }
                    Err(e) => {
                        match comment.as_mut() {
                            None => comment = Some(line.to_string()),
                            Some(comment) => writeln!(comment, "{}", line)?,
                        }
                        trace!("{e}")
                    }
                },
            }
        }
        Ok(items)
    }

    /// provider 中的规则没有 section 和 policy
    pub fn parse_rules_for_provider(lines: impl IntoIterator<Item = impl AsRef<str>>) -> ParseResult<Vec<Rule>> {
        let rules = Self::parse_comment(
            lines,
            |line| {
                let line = Self::trim_line_comment(line.as_ref());
                let fields = line.split(',').collect::<Vec<_>>();
                match fields.len() {
                    2 => {
                        let rule_type = RuleType::from_str(fields[0].trim())?;
                        let value = fields[1].trim().to_string();

                        Ok(Rule {
                            rule_type,
                            value: Some(value),
                            policy: Policy::default(),
                            comment: None,
                        })
                    }
                    _ => {
                        return Err(ParseError::Rule {
                            line: 0,
                            reason: format!("规则格式错误, 应该为`type,value[,policy[,option]]`: {line}"),
                        });
                    }
                }
            },
            |rule, comment| {
                rule.set_comment(comment);
            },
        )?;
        Ok(rules)
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

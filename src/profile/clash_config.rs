use crate::profile::rule::Rule;
use crate::profile::Proxy;

pub struct ClashConfig {
    pub proxies: Vec<Proxy>,
    pub rules: Vec<Rule>,
}

use std::fmt::{Display, Formatter};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantArray};
use url::Url;

#[derive(Debug, Clone)]
pub struct ConvertorUrl {
    pub r#type: ConvertorUrlType,
    pub server: Url,
    pub path: String,
    pub query: String,
}

impl From<&ConvertorUrl> for Url {
    fn from(value: &ConvertorUrl) -> Self {
        let mut url = value.server.clone();
        url.set_path(&value.path);
        url.set_query(Some(&value.query));
        url
    }
}

impl Display for ConvertorUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Url::from(self))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Display, IntoStaticStr, AsRefStr, VariantArray, EnumString)]
pub enum ConvertorUrlType {
    Raw,
    RawProfile,
    Profile,
    RuleProvider,
    SubLogs,
}

impl ConvertorUrlType {
    pub fn label(&self) -> &str {
        match self {
            ConvertorUrlType::Raw => "原始订阅链接",
            ConvertorUrlType::RawProfile => "非转换配置订阅链接",
            ConvertorUrlType::Profile => "转换配置订阅链接",
            ConvertorUrlType::RuleProvider => "规则集",
            ConvertorUrlType::SubLogs => "订阅日志链接",
        }
    }
}

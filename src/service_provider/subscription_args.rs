use crate::client::Client;
use clap::Args;
use url::Url;

#[derive(Debug, Args)]
pub struct SubscriptionArgs {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: Client,

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
    #[arg(short, long, default_value_t = 86400)]
    pub interval: u64,

    /// 是否严格模式
    /// 如果开启，订阅转换器将严格按照配置进行转换
    #[arg(short, long, default_value_t = true)]
    pub strict: bool,
}

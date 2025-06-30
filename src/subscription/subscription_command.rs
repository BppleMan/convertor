use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum SubscriptionCommand {
    /// 从 boslife 获取订阅地址
    Get,
    /// 从 boslife 更新订阅地址
    Update {
        /// 是否刷新 boslife token
        #[arg(short = 'r', long = "refresh", default_value = "false")]
        refresh_token: bool,
    },
    /// 将 boslife 的订阅地址编码为 convertor 的订阅地址
    Encode {
        /// boslife 的订阅地址
        #[arg(long = "url")]
        raw_subscription_url: String,
    },
    /// 将 convertor 的订阅地址解码为 boslife 的订阅地址
    Decode {
        /// convertor 的订阅地址
        #[arg(long = "url")]
        convertor_url: String,
    },
}

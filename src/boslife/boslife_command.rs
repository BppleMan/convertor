use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum BosLifeCommand {
    /// 从 boslife 获取订阅地址
    Get {
        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, long)]
        server: Option<String>,
    },
    /// 从 boslife 更新订阅地址
    Update {
        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, long)]
        server: Option<String>,

        /// 是否刷新 boslife token
        #[arg(short, long, default_value = "false")]
        refresh_token: bool,
    },
    /// 根据 boslife 的订阅地址编码为 convertor 的订阅地址
    Encode {
        /// convertor 所在服务器的地址
        /// 格式为 `http://ip:port`
        #[arg(short, long)]
        server: Option<String>,
        /// boslife 的订阅地址
        #[arg(short, long = "url")]
        raw_subscription_url: String,
    },
    /// 根据 convertor 的订阅地址解码为 boslife 的订阅地址
    Decode {
        /// convertor 的订阅地址
        #[arg(short, long = "url")]
        convertor_url: String,
    },
}

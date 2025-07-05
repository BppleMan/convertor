use crate::client::Client;
use clap::{Args, Subcommand};
use url::Url;

#[derive(Debug, Subcommand)]
pub enum SubscriptionCommand {
    /// 从服务商获取订阅地址
    Get(SubCommonArgs),
    /// 从服务商更新订阅地址
    Update {
        /// 是否刷新订阅 token
        #[arg(short = 'r', long = "reset", default_value = "false")]
        reset_token: bool,

        #[command(flatten)]
        args: SubCommonArgs,
    },
    /// 将服务商的订阅地址编码为 convertor 的订阅地址
    Encode {
        /// 服务商的订阅地址
        #[arg(long = "url")]
        raw_subscription_url: Url,

        #[command(flatten)]
        args: SubCommonArgs,
    },
    /// 将 convertor 的订阅地址解码为服务商的订阅地址
    Decode {
        /// convertor 的订阅地址
        #[arg(long = "url")]
        convertor_url: Url,

        #[command(flatten)]
        args: SubCommonArgs,
    },
}

impl SubscriptionCommand {
    pub fn args(&self) -> &SubCommonArgs {
        match self {
            SubscriptionCommand::Get(args) => args,
            SubscriptionCommand::Update { args, .. } => args,
            SubscriptionCommand::Encode { args, .. } => args,
            SubscriptionCommand::Decode { args, .. } => args,
        }
    }

    pub fn args_mut(&mut self) -> &mut SubCommonArgs {
        match self {
            SubscriptionCommand::Get(args) => args,
            SubscriptionCommand::Update { args, .. } => args,
            SubscriptionCommand::Encode { args, .. } => args,
            SubscriptionCommand::Decode { args, .. } => args,
        }
    }
}

#[derive(Debug, Args)]
pub struct SubCommonArgs {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: Client,

    /// convertor 所在服务器的地址
    /// 格式为 `http://ip:port`
    #[arg(short, long)]
    pub server: Option<Url>,
}

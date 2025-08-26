use clap::{Args, Subcommand};
use convertor::common::redis::{REDIS_CONVERTOR_CONFIG_KEY, REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL};
use convertor::config::ConvertorConfig;
use redis::{AsyncTypedCommands, Client};
use std::path::PathBuf;
use tracing::info;

#[derive(Default, Debug, Clone, Args)]
pub struct ConfigCmd {
    /// 配置文件路径
    #[arg()]
    file: Option<PathBuf>,

    /// 配置相关的子命令
    #[command(subcommand)]
    option: Option<ConfigCmdOption>,

    /// 是否发布配置到 Redis
    #[arg(short, long, default_value_t = false)]
    publish: bool,
}

#[derive(Default, Debug, Clone, Subcommand)]
pub enum ConfigCmdOption {
    #[default]
    /// 获取配置模板
    Template,

    /// 从 Redis 获取配置
    #[command(name = "redis")]
    Redis,

    /// 从文件获取配置
    #[command(name = "file")]
    File,
}

pub struct ConfigCli {
    pub cmd: ConfigCmd,
}

impl ConfigCli {
    pub fn new(cmd: ConfigCmd) -> Self {
        Self { cmd }
    }

    pub async fn execute(self, client: Client) -> color_eyre::Result<()> {
        let config = match (self.cmd.file, self.cmd.option) {
            (Some(file), _) => {
                let config = ConvertorConfig::from_file(file)?;
                println!("{config}");
                config
            }
            (None, Some(ConfigCmdOption::Template)) => {
                let config = ConvertorConfig::template();
                println!("{config}");
                config
            }
            (None, Some(ConfigCmdOption::Redis)) => {
                let connection = client.get_multiplexed_async_connection().await?;
                let config = ConvertorConfig::from_redis(connection).await?;
                println!("{config}");
                config
            }
            (None, Some(ConfigCmdOption::File)) => {
                let config = ConvertorConfig::search(std::env::current_dir()?, None::<&str>)?;
                println!("{config}");
                config
            }
            _ => return Ok(()),
        };
        if self.cmd.publish {
            info!("更新配置到 Redis");
            let mut connection = client.get_multiplexed_async_connection().await?;
            connection.set(REDIS_CONVERTOR_CONFIG_KEY, config.to_string()).await?;
            info!("发布配置更新到 Redis 频道");
            connection
                .publish(REDIS_CONVERTOR_CONFIG_PUBLISH_CHANNEL, REDIS_CONVERTOR_CONFIG_KEY)
                .await?;
        }
        Ok(())
    }
}

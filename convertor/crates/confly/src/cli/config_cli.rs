use clap::{Args, Subcommand};
use convertor::config::ConvertorConfig;
use std::path::PathBuf;
use color_eyre::eyre::eyre;

#[derive(Default, Debug, Clone, Args)]
pub struct ConfigCmd {
    /// 配置文件路径
    #[arg()]
    file: Option<PathBuf>,

    /// 配置相关的子命令
    #[command(subcommand)]
    option: Option<ConfigCmdOption>,
}

#[derive(Default, Debug, Clone, Subcommand)]
pub enum ConfigCmdOption {
    #[default]
    /// 获取配置模板
    Template,

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

    pub async fn execute(self) -> color_eyre::Result<ConvertorConfig> {
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
            (None, Some(ConfigCmdOption::File)) => {
                let config = ConvertorConfig::search(std::env::current_dir()?, None::<&str>)?;
                println!("{config}");
                config
            }
            _ => return Err(eyre!("必须指定配置文件路径或者子命令")),
        };
        Ok(config)
    }
}

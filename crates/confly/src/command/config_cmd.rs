use crate::config::ConflyConfig;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Subcommand)]
pub enum ConfigCmd {
    #[default]
    /// 获取配置模板
    #[command(name = "template")]
    Template,

    /// 验证现有配置
    #[command(name = "valid")]
    Validate,
}

impl ConfigCmd {
    pub async fn execute(self, base_dir: PathBuf, config: Option<PathBuf>) -> color_eyre::Result<ConflyConfig> {
        let config = match (self, config) {
            (ConfigCmd::Template, _) => {
                let config = ConflyConfig::template();
                println!("{config}");
                config
            }
            (ConfigCmd::Validate, file) => {
                let config = match file {
                    None => ConflyConfig::search(base_dir, None::<&str>)?,
                    Some(file) => ConflyConfig::from_file(file)?,
                };
                println!("{config}");
                config
            }
        };
        Ok(config)
    }
}

use crate::args::Profile;
use clap::{Args, ValueEnum};

#[derive(Debug, Args)]
pub struct VarCommand {
    /// 变量名
    #[arg(value_enum)]
    pub key: VarKey,

    /// Profile 配置 (dev/prod)
    #[arg(value_enum)]
    pub profile: Profile,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum VarKey {
    CargoProfile,
    CargoTargetDir,
    DashboardProfile,
}

impl VarCommand {
    pub fn run(&self) -> color_eyre::Result<()> {
        let value = match self.key {
            VarKey::CargoProfile => self.profile.as_cargo_profile().to_string(),
            VarKey::CargoTargetDir => self.profile.as_cargo_target_dir().to_string(),
            VarKey::DashboardProfile => self.profile.as_dashboard_profile().to_string(),
        };
        println!("{}", value);
        Ok(())
    }
}

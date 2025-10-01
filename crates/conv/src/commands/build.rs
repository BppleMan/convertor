use crate::args::{Package, Target};
use crate::commands::{Commander, DashboardCommand};
use crate::conv_cli::CommonArgs;
use clap::Args;
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct BuildCommand {
    #[command(flatten)]
    pub common_args: CommonArgs,

    /// 编译目标
    #[command(subcommand)]
    pub target: Option<Target>,

    /// 是否打包 dashboard
    #[arg(short, long)]
    pub dashboard: bool,
}

impl Commander for BuildCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let CommonArgs { profile, package } = &self.common_args;
        let target = self.target.clone().unwrap_or(Target::Native);
        let mut command = Command::new("cargo");
        command
            .arg(target.as_builder())
            .arg("--package")
            .arg(package.to_string())
            .arg("--profile")
            .arg(profile.as_cargo_profile());
        if let Some(target_triple) = target.as_target_triple() {
            command.arg("--target").arg(target_triple);
        }

        let mut commands = vec![];
        if matches!(package, Package::Convd) && self.dashboard {
            commands = DashboardCommand::new(profile.clone()).create_command()?;
        }
        commands.push(command);

        Ok(commands)
    }
}

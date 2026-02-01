use crate::args::{Package, Profile};
use crate::commands::{BuildCommand, Commander, DashboardCommand, ImageCommand, PublishCommand, VarCommand};
use clap::{Args, Parser, Subcommand};
use std::process::Command;

#[derive(Debug, Parser)]
pub struct ConvCli {
    #[command(subcommand)]
    pub command: ConvCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConvCommand {
    /// 显示版本信息
    Version,

    /// 编译 convertor
    Build(BuildCommand),

    /// 发布 convertor
    Publish(PublishCommand),

    /// 构建 convd 镜像
    Image(ImageCommand),

    /// 编译 dashboard
    Dashboard(DashboardCommand),

    /// 获取配置变量值
    Var(VarCommand),
}

#[derive(Debug, Args)]
pub struct CommonArgs {
    /// 需要构建的包
    #[arg(value_enum, default_value_t = Package::Convd)]
    pub package: Package,

    /// 构建配置
    #[arg(value_enum, default_value_t = Profile::Dev)]
    pub profile: Profile,
}

impl Commander for ConvCommand {
    fn create_command(&self) -> color_eyre::Result<Vec<Command>> {
        match self {
            ConvCommand::Version => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                Ok(vec![])
            }
            ConvCommand::Var(var) => {
                var.run()?;
                Ok(vec![])
            }
            ConvCommand::Build(build) => build.create_command(),
            ConvCommand::Publish(publish) => publish.create_command(),
            ConvCommand::Image(image) => image.create_command(),
            ConvCommand::Dashboard(dashboard) => dashboard.create_command(),
        }
    }
}

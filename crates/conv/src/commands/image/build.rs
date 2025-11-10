use crate::args::{Arch, Package, Tag, Target};
use crate::commands::image::BuildArgs;
use crate::commands::{BuildCommand, Commander, ImageCommonArgs};
use crate::conv_cli::CommonArgs;
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImageBuildCommand {
    #[clap(flatten)]
    common_args: ImageCommonArgs,

    #[arg(short, long, default_value_t = false)]
    dashboard: bool,
}

impl ImageBuildCommand {
    pub fn prepare(&self) -> color_eyre::Result<Vec<Command>> {
        BuildCommand {
            common_args: CommonArgs {
                profile: self.common_args.profile,
                package: Package::Convd,
            },
            target: Some(Target::Musl {
                arch: self.common_args.arch.clone(),
            }),
            dashboard: self.dashboard,
        }
        .create_command()
    }

    fn build_image(&self, tag: &Tag, arch: Arch) -> Command {
        let build_args = BuildArgs::new(arch, self.common_args.profile);
        let mut command = Command::new("docker");
        command
            .args(["buildx", "build"])
            .args(["--platform", arch.as_image_platform()])
            .args(["-t", tag.local(Some(arch)).as_str()]);
        build_args.build_arg(&mut command);
        command.args(["-f", "Dockerfile", "--load", "."]);

        command
    }
}

impl Commander for ImageBuildCommand {
    fn create_command(&self) -> color_eyre::Result<Vec<Command>> {
        let mut commands = self.prepare()?;
        let tag = Tag::new(&self.common_args.user, &self.common_args.project, self.common_args.profile);

        // 先将所有架构的镜像构建出来
        for arch in self.common_args.arch.iter().copied() {
            commands.push(self.build_image(&tag, arch));
        }

        Ok(commands)
    }
}

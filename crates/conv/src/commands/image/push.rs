use crate::args::{Arch, Registry, Tag};
use crate::commands::{Commander, ImageCommonArgs};
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImagePushCommand {
    #[clap(flatten)]
    common_args: ImageCommonArgs,

    #[arg(short, long, value_delimiter = ',', default_values_t = default_registries())]
    registries: Vec<Registry>,

    #[arg(long, alias = "cr", value_delimiter = ',')]
    custom_registries: Vec<String>,
}

impl ImagePushCommand {
    #[allow(unused)]
    fn tag_image(&self, tag: &Tag, registry: &Registry, arch: Arch) -> Command {
        let mut command = Command::new("docker");
        command.arg("tag").arg(tag.local(Some(arch))).arg(tag.remote(registry, Some(arch)));

        command
    }

    fn push_image(&self, tag: &Tag, registry: &Registry, arch: Arch) -> Command {
        let mut command = Command::new("skopeo");
        command
            .arg("copy")
            .arg(format!("docker-daemon:{}", tag.local(Some(arch))))
            .arg(format!("docker://{}", tag.remote(registry, Some(arch))));

        command
    }

    fn manifest_image(&self, tag: &Tag, registry: &Registry) -> Command {
        let mut command = Command::new("docker");
        command
            .args(["buildx", "imagetools", "create"])
            .args(["-t", tag.remote(registry, None).as_str()]);
        for arch in self.common_args.arch.iter().copied() {
            command.arg(tag.remote(registry, Some(arch)));
        }

        command
    }
}

impl Commander for ImagePushCommand {
    fn create_command(&self) -> color_eyre::Result<Vec<Command>> {
        let mut commands = vec![];
        let tag = Tag::new(&self.common_args.user, &self.common_args.project, self.common_args.profile);

        // 然后以注册表为单位，给每个架构的镜像打标签并推送
        let mut registries = self.registries.clone();
        for cr in &self.custom_registries {
            registries.push(Registry::Custom(cr.to_string()));
        }
        for registry in registries {
            for arch in self.common_args.arch.iter().copied() {
                commands.push(self.push_image(&tag, &registry, arch));
            }
            // 最后创建多架构清单并推送
            commands.push(self.manifest_image(&tag, &registry));
        }

        Ok(commands)
    }
}

fn default_registries() -> Vec<Registry> {
    vec![Registry::Ghcr]
}

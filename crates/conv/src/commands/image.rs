use crate::args::{Arch, Package, Profile, Registry, Tag, Target};
use crate::commands::{BuildCommand, Commander};
use crate::conv_cli::CommonArgs;
use clap::Args;
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImageCommand {
    /// 指定编译 profile
    #[arg(value_enum)]
    pub profile: Profile,

    /// 指定编译架构
    #[arg(short, long, value_delimiter = ',', default_values_t = default_arch())]
    pub arch: Vec<Arch>,

    /// 指定镜像注册表用户名
    #[arg(long, default_value_t = default_user())]
    pub user: String,

    /// 指定镜像注册表项目名称
    #[arg(long, default_value_t = default_project())]
    pub project: String,

    /// 指定镜像注册表
    #[arg(short, long, value_delimiter = ',')]
    registries: Vec<Registry>,

    /// 指定自定义镜像注册表地址
    #[arg(long, alias = "cr", value_delimiter = ',')]
    custom_registries: Vec<String>,

    /// 是否打包 dashboard
    #[arg(short, long, default_value_t = false)]
    dashboard: bool,

    /// 是否仅推送
    #[arg(long, alias = "po", default_value_t = false)]
    push_only: bool,
}

impl ImageCommand {
    pub fn build(&self) -> color_eyre::Result<Vec<Command>> {
        BuildCommand {
            common_args: CommonArgs {
                profile: self.profile,
                package: Package::Convd,
            },
            target: Some(Target::Musl { arch: self.arch.clone() }),
            dashboard: self.dashboard,
        }
            .create_command()
    }

    fn build_image(&self, tag: &Tag, arch: Arch) -> Command {
        let build_args = BuildArgs::new(arch, self.profile);
        let mut command = Command::new("docker");
        command
            .args(["buildx", "build"])
            .args(["--platform", arch.as_image_platform()])
            .args(["-t", tag.local(Some(arch)).as_str()]);
        build_args.build_arg(&mut command);
        command.args(["-f", "Dockerfile", "--load", "."]);

        command
    }

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
        for arch in self.arch.iter().copied() {
            command.arg(tag.remote(registry, Some(arch)));
        }

        command
    }
}

impl Commander for ImageCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let mut commands = vec![];
        let tag = Tag::new(&self.user, &self.project, self.profile);

        if !self.push_only {
            commands.extend(self.build()?);
            // 先将所有架构的镜像构建出来
            for arch in self.arch.iter().copied() {
                commands.push(self.build_image(&tag, arch));
            }
        }

        // 然后以注册表为单位，给每个架构的镜像打标签并推送
        let mut registries = self.registries.clone();
        for cr in &self.custom_registries {
            registries.push(Registry::Custom(cr.to_string()));
        }
        for registry in registries {
            for arch in self.arch.iter().copied() {
                commands.push(self.push_image(&tag, &registry, arch));
            }
            // 最后创建多架构清单并推送
            commands.push(self.manifest_image(&tag, &registry));
        }

        Ok(commands)
    }
}

struct BuildArgs {
    name: String,
    version: String,
    description: String,
    url: String,
    vendor: String,
    license: String,
    build_date: String,

    target_triple: String,
    target_dir: String,
}

impl BuildArgs {
    fn new(arch: Arch, profile: Profile) -> Self {
        let name = "convd".to_string();
        let version = env!("CARGO_PKG_VERSION").to_string();
        let description = env!("CARGO_PKG_DESCRIPTION").to_string();
        let url = env!("CARGO_PKG_REPOSITORY").to_string();
        let vendor = env!("CARGO_PKG_AUTHORS").to_string().replace("[", "").replace("]", "");
        let license = env!("CARGO_PKG_LICENSE").to_string();
        let build_date = chrono::Utc::now().to_rfc3339();

        let target_triple = arch.as_target_triple().to_string();
        let target_dir = profile.as_cargo_target_dir().to_string();
        Self {
            name,
            version,
            description,
            url,
            vendor,
            license,
            build_date,
            target_triple,
            target_dir,
        }
    }

    pub fn build_arg(&self, command: &mut Command) {
        command.arg("--build-arg").arg(format!("NAME={}", self.name));
        command.arg("--build-arg").arg(format!("VERSION={}", self.version));
        command.arg("--build-arg").arg(format!("DESCRIPTION={}", self.description));
        command.arg("--build-arg").arg(format!("URL={}", self.url));
        command.arg("--build-arg").arg(format!("VENDOR={}", self.vendor));
        command.arg("--build-arg").arg(format!("LICENSE={}", self.license));
        command.arg("--build-arg").arg(format!("BUILD_DATE={}", self.build_date));
        command.arg("--build-arg").arg(format!("TARGET_TRIPLE={}", self.target_triple));
        command.arg("--build-arg").arg(format!("TARGET_DIR={}", self.target_dir));
    }
}

fn default_arch() -> Vec<Arch> {
    vec![Arch::current()]
}

fn default_user() -> String {
    "bppleman".to_string()
}

fn default_project() -> String {
    "convertor".to_string()
}

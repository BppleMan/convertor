use crate::args::{Arch, Package, Profile, Registry, Tag, Target, Version};
use crate::commands::{BuildCommand, Commander};
use crate::conv_cli::CommonArgs;
use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::bail;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImageCommand {
    /// 指定编译 profile
    #[arg(value_enum)]
    pub profile: Profile,

    /// 指定编译架构
    #[arg(short, long, value_delimiter = ',')]
    pub arch: Vec<Arch>,

    #[arg(short, long, alias = "ver", default_value_t = default_version())]
    pub version: Version,

    /// 指定镜像注册表用户名
    #[arg(long, default_value_t = default_user())]
    pub user: String,

    /// 指定镜像注册表项目名称
    #[arg(long, default_value_t = default_project())]
    pub project: String,

    /// 指定镜像注册表，[local, docker, ghcr, custom_url]
    #[arg(short, long, value_enum, value_delimiter = ',')]
    registries: Vec<Registry>,

    // /// 指定自定义镜像注册表地址
    // #[arg(long, alias = "cr", value_delimiter = ',')]
    // custom_registries: Vec<String>,
    /// 是否打包 dashboard
    #[arg(short, long, default_value_t = false)]
    dashboard: bool,

    /// 是否仅推送
    #[arg(long, alias = "po", default_value_t = false)]
    push_only: bool,

    /// 仅打印镜像标签（不构建）
    #[arg(short, long, alias = "to", default_value_t = false)]
    tag: bool,
}

impl ImageCommand {
    pub fn build(&self) -> Result<Vec<Command>> {
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
            .args(["-t", tag.local(Some(arch), None).as_str()]);
        build_args.build_arg(&mut command);
        command.args(["-f", "Dockerfile", "--load", "."]);

        command
    }

    fn push_image(&self, tag: &Tag, registry: &Registry, arch: Arch) -> Command {
        let mut command = Command::new("skopeo");
        command
            .arg("copy")
            .arg(format!("docker-daemon:{}", tag.local(Some(arch), None)))
            .arg(format!("docker://{}", tag.remote(registry, Some(arch), None)));

        command
    }

    fn manifest_image(&self, tag: &Tag, registry: &Registry, version: &Version) -> Command {
        let mut command = Command::new("docker");
        command
            .args(["buildx", "imagetools", "create"])
            .args(["-t", tag.remote(registry, None, Some(version)).as_str()]);
        for arch in self.arch.iter().copied() {
            command.arg(tag.remote(registry, Some(arch), None));
        }

        command
    }
}

impl Commander for ImageCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let tag = Tag::new(&self.user, &self.project, self.version.clone(), self.profile);

        // 仅打印标签
        if self.tag {
            let Some(registry) = self.registries.first() else {
                bail!("--tag 需要指定至少一个 --registry");
            };
            println!("{}", tag.remote(registry, self.arch.first().copied(), Some(&self.version)));
            return Ok(vec![]);
        }

        let mut commands = vec![];

        if !self.push_only {
            commands.extend(self.build()?);
            // 先将所有架构的镜像构建出来
            for arch in self.arch.iter().copied() {
                commands.push(self.build_image(&tag, arch));
            }
        }

        // 然后以注册表为单位，给每个架构的镜像打标签并推送
        for registry in &self.registries {
            for arch in self.arch.iter().copied() {
                commands.push(self.push_image(&tag, registry, arch));
            }
            // 最后创建多架构清单并推送，需要包含version标签和latest标签
            for version in [&self.version, &Version::Latest] {
                commands.push(self.manifest_image(&tag, registry, version));
            }
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

fn default_user() -> String {
    "convertor-gitops".to_string()
}

fn default_project() -> String {
    "convertor".to_string()
}

fn default_version() -> Version {
    Version::Specific(env!("CARGO_PKG_VERSION").to_string())
}

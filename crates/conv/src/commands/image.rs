use crate::args::{Arch, Package, Profile, Registry, Tag, Target};
use crate::commands::{BuildCommand, Commander};
use crate::conv_cli::CommonArgs;
use clap::Args;
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImageCommand {
    #[arg(value_enum)]
    pub profile: Profile,

    #[arg(short, long, value_delimiter = ',', default_values_t = default_arch())]
    pub arch: Vec<Arch>,

    #[arg(short, long, value_delimiter = ',', default_values_t = default_registries())]
    pub registries: Vec<Registry>,

    #[arg(long, default_value_t = default_user())]
    pub user: String,

    #[arg(long, default_value_t = default_project())]
    pub project: String,

    #[arg(short, long, default_value_t = false)]
    pub push: bool,

    #[arg(short, long, default_value_t = false)]
    pub dashboard: bool,
}

impl ImageCommand {
    pub fn prepare(&self) -> Result<Vec<Command>> {
        BuildCommand {
            common_args: CommonArgs {
                profile: self.profile.clone(),
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
    fn tag_image(&self, tag: &Tag, registry: Registry, arch: Arch) -> Command {
        let mut command = Command::new("docker");
        command.arg("tag").arg(tag.local(Some(arch))).arg(tag.remote(registry, Some(arch)));

        command
    }

    fn push_image(&self, tag: &Tag, registry: Registry, arch: Arch) -> Command {
        let mut command = Command::new("skopeo");
        command
            .arg("copy")
            .arg(format!("docker-daemon:{}", tag.local(Some(arch))))
            .arg(format!("docker://{}", tag.remote(registry, Some(arch))));

        command
    }

    fn manifest_image(&self, tag: &Tag, registry: Registry) -> Command {
        let mut command = Command::new("docker");
        command
            .args(["buildx", "imagetools", "create"])
            .args(["-t", tag.remote(registry, None).as_str()]);
        for arch in self.arch.iter().copied() {
            command.arg(tag.remote(registry, Some(arch)));
        }

        command
    }

    fn login_registry(&self, registry: Registry) -> Command {
        let echo_token = format!(r#"echo "${}{}_TOKEN{}""#, "{", registry.env_prefix(), "}");
        let login = format!(
            r#"login -u "${}{}_USER{}" --password-stdin {}"#,
            "{",
            registry.env_prefix(),
            "}",
            registry.as_url()
        );
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(format!(r#"{} | docker {} && {} | skopeo {}"#, echo_token, login, echo_token, login));

        command
    }
}

impl Commander for ImageCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let mut commands = self.prepare()?;

        let tag = Tag::new(&self.user, &self.project, self.profile);
        // 先将所有架构的镜像构建出来
        for arch in self.arch.iter().copied() {
            commands.push(self.build_image(&tag, arch));
        }

        // 然后以注册表为单位，给每个架构的镜像打标签并推送
        for registry in self.registries.iter().copied() {
            // 本地注册表不需要打标签和推送
            if registry == Registry::Local {
                continue;
            }
            // 每个注册表需要单独登录
            commands.push(self.login_registry(registry));
            for arch in self.arch.iter().copied() {
                commands.push(self.push_image(&tag, registry, arch));
            }
            // 最后创建多架构清单并推送
            commands.push(self.manifest_image(&tag, registry));
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

fn default_registries() -> Vec<Registry> {
    vec![Registry::Local, Registry::Ghcr, Registry::Harbor]
}

fn default_user() -> String {
    "bppleman".to_string()
}

fn default_project() -> String {
    "convertor".to_string()
}

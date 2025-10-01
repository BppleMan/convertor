use crate::args::{Arch, Package, Profile, Target};
use crate::commands::{BuildCommand, Commander};
use crate::conv_cli::CommonArgs;
use clap::Args;
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImageCommand {
    #[arg(value_enum)]
    pub profile: Profile,

    #[arg(short, long, default_value_t = false)]
    pub multi: bool,

    #[arg(short, long, default_value_t = false)]
    pub dashboard: bool,
}

impl ImageCommand {
    pub fn prepare(&self) -> Result<Vec<Command>> {
        let mut build_command = BuildCommand {
            common_args: CommonArgs {
                profile: self.profile.clone(),
                package: Package::Convd,
            },
            target: Some(Target::Musl { arch: Arch::Amd }),
            dashboard: self.dashboard,
        };
        let mut pre_commands = build_command.create_command()?;

        build_command.target = Some(Target::Musl { arch: Arch::Arm });
        pre_commands.extend(build_command.create_command()?);

        Ok(pre_commands)
    }

    // pub fn build(&self) -> Result<Vec<Command>> {
    //     let mut commands = self.prepare()?;
    //
    //     let registry = self.profile.as_image_registry();
    //     let docker_args = DockerArgs::new(&self.profile, &self.arch);
    //     let tag = format!("{}/{}:{}", registry, docker_args.name, docker_args.version);
    //
    //     let mut command = Command::new("docker");
    //     command
    //         .args(["buildx", "build"])
    //         .args(["--platform", self.arch.as_image_platform()])
    //         .args(["-f", "Dockerfile"]);
    //     docker_args.build_arg(&mut command);
    //     command.arg("--load");
    //     command.args(["-t", tag.as_str()]).arg(".");
    //     commands.push(command);
    //
    //     Ok(commands)
    // }
    //
    // pub fn merge(&self) -> Result<Vec<Command>> {
    //     if matches!(self.profile, Profile::Dev) {
    //         return Err(eyre!("不能在 dev 配置下合并镜像"));
    //     }
    //     let registry = self.profile.as_image_registry();
    //     let docker_args = DockerArgs::new(&self.profile, &self.arch);
    //
    //     let mut command = Command::new("docker");
    //     command
    //         .args(["buildx", "imagetools", "create"])
    //         .arg("-t")
    //         .arg(format!("{}/{}:{}", registry, docker_args.name, docker_args.version))
    //         .arg("-t")
    //         .arg(format!("{}/{}:latest", registry, docker_args.name))
    //         .args([
    //             format!(
    //                 "{}/{}-{}:{}",
    //                 registry,
    //                 docker_args.name,
    //                 Arch::Arm,
    //                 docker_args.version
    //             ),
    //             format!(
    //                 "{}/{}-{}:{}",
    //                 registry,
    //                 docker_args.name,
    //                 Arch::Amd,
    //                 docker_args.version
    //             ),
    //         ]);
    //
    //     Ok(vec![command])
    // }

    fn copy_command(&self, docker_args: &DockerArgs, arch: Arch) -> Command {
        let bin_path = format!(
            "target/{}/{}/{}",
            arch.as_target_triple(),
            self.profile.as_cargo_target_dir(),
            docker_args.name
        );
        let dist_path = format!("{}/{}", arch.as_dist_path(), docker_args.name);
        let mut copy = Command::new("sh");
        copy.arg("-c")
            .arg(format!("mkdir -p {} && cp -rf {} {}", arch.as_dist_path(), bin_path, dist_path));

        copy
    }
}

impl Commander for ImageCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let mut commands = self.prepare()?;

        let multi = self.multi && matches!(self.profile, Profile::Prod);
        let registry = self.profile.as_image_registry();
        let docker_args = DockerArgs::new();
        let tag = format!("{}/{}:{}", registry, docker_args.name, docker_args.version);
        let (load_or_push, platform) = match multi {
            false => ("--load", Arch::current().as_image_platform().to_string()),
            true => (
                "--push",
                format!("{},{}", Arch::Amd.as_image_platform(), Arch::Arm.as_image_platform()),
            ),
        };

        let mut command = Command::new("docker");
        command
            .args(["buildx", "build"])
            .args(["--platform", platform.as_str()])
            .args(["-f", "Dockerfile"]);
        docker_args.build_arg(&mut command);
        command.args(["-t", tag.as_str()]).arg(load_or_push).arg(".");

        if !multi {
            commands.push(self.copy_command(&docker_args, Arch::current()));
        } else {
            commands.push(self.copy_command(&docker_args, Arch::Amd));
            commands.push(self.copy_command(&docker_args, Arch::Arm));
        }
        commands.push(command);

        Ok(commands)
    }
}

struct DockerArgs {
    name: String,
    version: String,
    description: String,
    url: String,
    vendor: String,
    license: String,
    build_date: String,
}

impl DockerArgs {
    fn new() -> Self {
        let name = "convd".to_string();
        let version = env!("CARGO_PKG_VERSION").to_string();
        let description = env!("CARGO_PKG_DESCRIPTION").to_string();
        let url = env!("CARGO_PKG_REPOSITORY").to_string();
        let vendor = env!("CARGO_PKG_AUTHORS").to_string().replace("[", "").replace("]", "");
        let license = env!("CARGO_PKG_LICENSE").to_string();
        let build_date = chrono::Utc::now().to_rfc3339();

        Self {
            name,
            version,
            description,
            url,
            vendor,
            license,
            build_date,
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
    }
}

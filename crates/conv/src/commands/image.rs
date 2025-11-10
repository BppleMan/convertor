mod build;
mod push;

use crate::args::{Arch, Profile};
use crate::commands::Commander;
use crate::commands::image::build::ImageBuildCommand;
use crate::commands::image::push::ImagePushCommand;
use clap::{Args, Subcommand};
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Subcommand)]
pub enum ImageCommand {
    Build(ImageBuildCommand),

    Push(ImagePushCommand),
}

#[derive(Debug, Args)]
pub struct ImageCommonArgs {
    #[arg(value_enum)]
    pub profile: Profile,

    #[arg(short, long, value_delimiter = ',', default_values_t = default_arch())]
    pub arch: Vec<Arch>,

    #[arg(long, default_value_t = default_user())]
    pub user: String,

    #[arg(long, default_value_t = default_project())]
    pub project: String,
}

impl Commander for ImageCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        match self {
            ImageCommand::Build(build) => build.create_command(),
            ImageCommand::Push(push) => push.create_command(),
        }
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

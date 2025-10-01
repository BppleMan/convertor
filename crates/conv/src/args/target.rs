use crate::args::arch::Arch;
use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum Target {
    /// 编译本机目标
    Native,
    /// 编译 linux-musl 目标
    Musl {
        #[arg(value_enum, default_value_t = Arch::current())]
        arch: Arch,
    },
}

impl Target {
    pub fn as_builder(&self) -> &'static str {
        match self {
            Target::Native => "build",
            Target::Musl { .. } => "zigbuild",
        }
    }

    pub fn as_target_triple(&self) -> Option<&'static str> {
        match self {
            Target::Native => None,
            Target::Musl { arch } => Some(arch.as_target_triple()),
        }
    }
}

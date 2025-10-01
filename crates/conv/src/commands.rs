mod build;
mod dashboard;
mod image;
mod publish;

pub use build::*;
use color_eyre::Result;
pub use dashboard::*;
pub use image::*;
pub use publish::*;
use std::process::Command;

pub trait Commander {
    fn create_command(&self) -> Result<Vec<Command>>;
}

pub fn pretty_command(command: &Command) -> String {
    let mut cmd = command.get_program().to_string_lossy().to_string();
    for arg in command.get_args() {
        cmd.push(' ');
        cmd.push_str(&arg.to_string_lossy());
    }
    cmd
}

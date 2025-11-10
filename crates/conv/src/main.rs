use crate::commands::{Commander, pretty_command};
use crate::conv_cli::ConvCli;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use console::style;

mod args;
mod commands;
mod conv_cli;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = ConvCli::parse();
    let mut commands = cli.command.create_command()?;
    let len = commands.len();
    for (i, command) in commands.iter_mut().enumerate() {
        let command_str = pretty_command(command);
        let instant = std::time::Instant::now();
        println!(
            "{} {}",
            style(format!("[{i}/{len}]")).green(),
            style(command_str).blue().bold().italic()
        );
        let status = command.status()?;
        if !status.success() {
            let message = format!("命令执行失败: {}, 状态: {}", pretty_command(command), status);
            return Err(eyre!("{}", message));
        }
        let elapsed = instant.elapsed();
        println!("{} {}", style("[完成]").green(), style(format!("耗时: {:.2?}", elapsed)).purple());
    }

    Ok(())
}

// fn init_log() {
//     tracing_subscriber::registry()
//         .with(
//             tracing_subscriber::fmt::layer()
//                 .with_target(true)
//                 .with_level(true)
//                 .with_file(false)
//                 .with_line_number(false)
//                 .with_thread_names(false)
//                 .with_ansi(std::io::stdout().is_terminal())
//                 .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
//                 .pretty()
//                 .compact(),
//         )
//         .init();
// }

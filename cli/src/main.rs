mod boards;
mod commands;
mod config;
mod tools;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rvkit")]
#[command(version)]
#[command(about = "Bare metal Zig, without the bare metal pain.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new project for a target board
    New {
        #[arg(long, short)]
        board: String,
        name: String,
    },
    /// Build the project
    Build,
    /// Check local rvkit toolchain dependencies
    Doctor,
    /// Flash the firmware onto the board
    Flash,
    /// Serial monitor
    Monitor {
        /// Serial port (e.g. /dev/ttyUSB0, COM3); defaults to [flash] port in rvkit.toml
        #[arg(long, short)]
        port: Option<String>,
        /// Baud rate; defaults to [flash] baud_rate in rvkit.toml, else 115200
        #[arg(long, short)]
        baud: Option<u32>,
    },
    /// List supported boards
    Boards,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { board, name } => commands::new::run(&board, &name),
        Commands::Boards => commands::boards::run(),
        Commands::Build => commands::build::run(),
        Commands::Doctor => commands::doctor::run(),
        Commands::Flash => commands::flash::run(),
        Commands::Monitor { port, baud } => commands::monitor::run(port.as_deref(), baud),
    }
}

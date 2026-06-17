use crate::config;
use std::path::Path;
use std::process::Command;

pub fn run() {
    let config = config::load();

    let board = crate::boards::get(&config.project.board).unwrap_or_else(|| {
        eprintln!("Board '{}' is not supported.", config.project.board);
        std::process::exit(1);
    });

    if board.experimental {
        eprintln!(
            "Error: '{}' support is experimental — flashing is not implemented yet.",
            board.name
        );
        eprintln!(
            "       Flashing this board safely requires the {} image pipeline; see the roadmap in the README.",
            board.flash_tool
        );
        std::process::exit(1);
    }

    let binary = format!("zig-out/bin/{}", config.project.name);
    if !Path::new(&binary).exists() {
        eprintln!("Error: '{}' not found. Run 'rvkit build' first.", binary);
        std::process::exit(1);
    }

    println!("Flashing '{}' onto board '{}'...", binary, board.name);

    let status = match board.flash_tool {
        "wlink" => Command::new("wlink").args(["flash", &binary]).status(),
        _ => {
            eprintln!("Flash tool '{}' is not supported.", board.flash_tool);
            std::process::exit(1);
        }
    };

    match status {
        Ok(s) if s.success() => println!("✓ Flash succeeded!"),
        Ok(_) => {
            eprintln!("✗ Flash failed.");
            std::process::exit(1);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "Error: '{}' not found. Install it and try again.",
                board.flash_tool
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: failed to run '{}': {}", board.flash_tool, e);
            std::process::exit(1);
        }
    }
}

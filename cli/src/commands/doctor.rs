use std::path::Path;
use std::process::{Command, Stdio};

pub fn run() {
    println!("Checking rvkit environment...\n");

    let mut missing = false;

    if check_tool("zig", ["version"]) {
        println!("ok   zig found");
    } else {
        println!("miss zig not found");
        println!("     Install Zig from https://ziglang.org/download/");
        missing = true;
    }

    if check_tool("wlink", ["--version"]) {
        println!("ok   wlink found");
    } else {
        println!("warn wlink not found");
        println!("     Required to flash CH32V003 boards");
    }

    if check_tool("esptool.py", ["version"]) || check_tool("esptool", ["version"]) {
        println!("ok   esptool found");
    } else {
        println!("warn esptool not found");
        println!("     Required to flash ESP32-C3 boards");
    }

    if Path::new("rvkit.toml").exists() {
        println!("ok   rvkit.toml found");
    } else {
        println!("info rvkit.toml not found in current directory");
        println!("     Run doctor inside an rvkit project for project-specific checks");
    }

    if missing {
        eprintln!("\nDoctor found missing required tools.");
        std::process::exit(1);
    }

    println!("\nDoctor completed.");
}

fn check_tool<const N: usize>(program: &str, args: [&str; N]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

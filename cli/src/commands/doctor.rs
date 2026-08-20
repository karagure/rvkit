use crate::tools;
use std::path::Path;
use std::process::{Command, Stdio};

/// Generated build.zig files use the `root_module`/`createModule` API and
/// `callconv(.c)`, which require Zig 0.14 or newer.
const MIN_ZIG: (u32, u32) = (0, 14);

pub fn run() {
    println!("Checking rvkit environment...\n");

    let mut missing = false;

    match zig_version() {
        Some(version) => match version_at_least(&version, MIN_ZIG) {
            Some(true) => println!("ok   zig {} found", version),
            Some(false) => {
                println!("miss zig {} is too old", version);
                println!(
                    "     rvkit projects need Zig {}.{} or newer — https://ziglang.org/download/",
                    MIN_ZIG.0, MIN_ZIG.1
                );
                missing = true;
            }
            None => println!(
                "warn zig found but version '{}' could not be parsed",
                version
            ),
        },
        None => {
            println!("miss zig not found");
            println!("     Install Zig from https://ziglang.org/download/");
            missing = true;
        }
    }

    if tools::exists("wlink", &["--version"]) {
        println!("ok   wlink found");
    } else {
        println!("warn wlink not found");
        println!("     Required to flash CH32V003 boards");
    }

    match tools::find_esptool() {
        Some(name) => println!("ok   esptool found (as '{}')", name),
        None => {
            println!("warn esptool not found");
            println!("     Required to flash ESP32-C3 boards");
        }
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

fn zig_version() -> Option<String> {
    let output = Command::new("zig")
        .arg("version")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Parses "major.minor[...]" (e.g. "0.15.2", "0.16.0-dev.1+abc") and compares.
fn version_at_least(version: &str, min: (u32, u32)) -> Option<bool> {
    let mut parts = version.split(['.', '-', '+']);
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    Some((major, minor) >= min)
}

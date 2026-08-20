use std::process::Command;

pub fn run() {
    println!("Building...");

    let status = Command::new("zig")
        .arg("build")
        .status()
        .unwrap_or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("Error: 'zig' not found. Install Zig from https://ziglang.org/download/");
            } else {
                eprintln!("Error: failed to run 'zig build': {}", e);
            }
            std::process::exit(1);
        });
    if status.success() {
        println!("Build succeeded!");
    } else {
        eprintln!("Build failed.");
        std::process::exit(1);
    }
}

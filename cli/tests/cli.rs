use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn rvkit() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rvkit"))
}

fn temp_workspace(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before UNIX_EPOCH")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rvkit-{name}-{unique}"));
    fs::create_dir_all(&path).expect("failed to create temp workspace");
    path
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("failed to read generated file")
}

#[test]
fn version_flag_prints_package_version() {
    let output = rvkit()
        .arg("--version")
        .output()
        .expect("failed to run rvkit --version");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("rvkit {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn boards_command_lists_supported_boards() {
    let output = rvkit()
        .arg("boards")
        .output()
        .expect("failed to run rvkit boards");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ch32v003"));
    assert!(stdout.contains("esp32-c3"));
    assert!(stdout.contains("wlink"));
    assert!(stdout.contains("esptool"));
}

#[test]
fn new_command_generates_ch32v003_project_with_entry_point() {
    let workspace = temp_workspace("new-ch32v003");
    let output = rvkit()
        .current_dir(&workspace)
        .args(["new", "--board", "ch32v003", "blink"])
        .output()
        .expect("failed to run rvkit new");

    assert!(
        output.status.success(),
        "rvkit new failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let project = workspace.join("blink");
    let main_zig = read(project.join("src/main.zig"));
    let linker = read(project.join("linker/ch32v003.ld"));
    let config = read(project.join("rvkit.toml"));
    let build_zig = read(project.join("build.zig"));

    assert!(main_zig.contains("export fn _start() callconv(.c) noreturn"));
    assert!(main_zig.contains("fn main() void"));
    assert!(linker.contains("ENTRY(_start)"));
    assert!(config.contains("name = \"blink\""));
    assert!(config.contains("board = \"ch32v003\""));
    assert!(build_zig.contains(".cpu_arch = .riscv32"));

    fs::remove_dir_all(workspace).expect("failed to clean temp workspace");
}

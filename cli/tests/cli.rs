use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn rvkit() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rvkit"))
}

/// Temp dir removed on drop, so it is cleaned up even when an assertion fails.
struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rvkit-{name}-{unique}"));
        fs::create_dir_all(&path).expect("failed to create temp workspace");
        Self { path }
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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
    assert!(stdout.contains("experimental"));
}

#[test]
fn new_command_generates_ch32v003_project_with_startup() {
    let workspace = TempWorkspace::new("new-ch32v003");
    let output = rvkit()
        .current_dir(&workspace.path)
        .args(["new", "--board", "ch32v003", "blink"])
        .output()
        .expect("failed to run rvkit new");

    assert!(
        output.status.success(),
        "rvkit new failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let project = workspace.path.join("blink");
    let main_zig = read(project.join("src/main.zig"));
    let start_zig = read(project.join("src/start.zig"));
    let linker = read(project.join("linker/ch32v003.ld"));
    let config = read(project.join("rvkit.toml"));
    let build_zig = read(project.join("build.zig"));

    assert!(main_zig.contains("pub fn main() void"));

    // Startup code: stack pointer, .data copy, .bss zeroing.
    assert!(start_zig.contains("export fn _start()"));
    assert!(start_zig.contains("la sp, _stack_top"));
    assert!(start_zig.contains("_sbss"));
    assert!(start_zig.contains("_sdata"));

    assert!(linker.contains("ENTRY(_start)"));
    assert!(linker.contains("ORIGIN = 0x08000000"));

    assert!(config.contains("name = \"blink\""));
    assert!(config.contains("board = \"ch32v003\""));

    // CH32V003 is RV32EC: the target must add `e`+`c` and drop `i`.
    assert!(build_zig.contains(".cpu_arch = .riscv32"));
    assert!(build_zig.contains(".cpu_features_add = std.Target.riscv.featureSet(&.{ .e, .c })"));
    assert!(build_zig.contains(".cpu_features_sub = std.Target.riscv.featureSet(&.{ .i })"));
    assert!(build_zig.contains("src/start.zig"));
}

#[test]
fn new_command_warns_for_experimental_board() {
    let workspace = TempWorkspace::new("new-esp32c3");
    let output = rvkit()
        .current_dir(&workspace.path)
        .args(["new", "--board", "esp32-c3", "wifi"])
        .output()
        .expect("failed to run rvkit new");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("experimental"));
    assert!(workspace.path.join("wifi/linker/esp32-c3.ld").exists());
}

#[test]
fn new_command_rejects_unknown_board() {
    let workspace = TempWorkspace::new("new-unknown-board");
    let output = rvkit()
        .current_dir(&workspace.path)
        .args(["new", "--board", "atmega328", "nope"])
        .output()
        .expect("failed to run rvkit new");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not supported"));
    assert!(!workspace.path.join("nope").exists());
}

#[test]
fn new_command_rejects_invalid_project_name() {
    let workspace = TempWorkspace::new("new-bad-name");
    let output = rvkit()
        .current_dir(&workspace.path)
        .args(["new", "--board", "ch32v003", "bad\"name"])
        .output()
        .expect("failed to run rvkit new");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid character"));
}

#[test]
fn flash_fails_outside_a_project() {
    let workspace = TempWorkspace::new("flash-no-project");
    let output = rvkit()
        .current_dir(&workspace.path)
        .arg("flash")
        .output()
        .expect("failed to run rvkit flash");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("rvkit.toml not found"));
}

#[test]
fn flash_refuses_experimental_board() {
    let workspace = TempWorkspace::new("flash-experimental");
    let scaffold = rvkit()
        .current_dir(&workspace.path)
        .args(["new", "--board", "esp32-c3", "wifi"])
        .output()
        .expect("failed to run rvkit new");
    assert!(scaffold.status.success());

    let output = rvkit()
        .current_dir(workspace.path.join("wifi"))
        .arg("flash")
        .output()
        .expect("failed to run rvkit flash");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("experimental"));
}

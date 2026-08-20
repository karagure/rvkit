use std::process::{Command, Stdio};

/// Returns true if `program` can be spawned (i.e. it exists on PATH).
pub fn exists(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// esptool is installed as `esptool` by recent pip releases and as
/// `esptool.py` by older ones; accept either.
pub fn find_esptool() -> Option<&'static str> {
    ["esptool", "esptool.py"]
        .into_iter()
        .find(|name| exists(name, &["version"]))
}

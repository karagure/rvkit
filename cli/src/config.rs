use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub project: Project,
    #[serde(default)]
    pub flash: Flash,
}

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    pub board: String,
}

#[derive(Deserialize, Default)]
pub struct Flash {
    pub port: Option<String>,
    pub baud_rate: Option<u32>,
}

/// Loads rvkit.toml from the current directory. Returns None if the file is
/// absent; exits with a parse error message if it is present but invalid.
pub fn try_load() -> Option<Config> {
    let content = fs::read_to_string("rvkit.toml").ok()?;
    match toml::from_str(&content) {
        Ok(config) => Some(config),
        Err(e) => {
            eprintln!("Error: rvkit.toml is invalid: {}", e);
            std::process::exit(1);
        }
    }
}

/// Loads rvkit.toml or exits if the current directory is not an rvkit project.
pub fn load() -> Config {
    try_load().unwrap_or_else(|| {
        eprintln!("Error: rvkit.toml not found. Are you inside an rvkit project?");
        std::process::exit(1);
    })
}

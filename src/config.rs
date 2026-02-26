use serde::Deserialize;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util::get_invoking_user_env;

pub const TEMP_CONFIG_PATH: &str = "/var/lib/forge/.tmp";

pub enum ConfigCommand {
    Build,
    Install,
    Uninstall,
}

#[derive(Deserialize)]
pub struct Config {
    pub update: Option<String>,
    pub build: Option<String>,
    pub install: Option<String>,
    pub uninstall: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

impl Config {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Option<Self> {
        let contents = match fs::read_to_string(filepath) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("no package config found");
                return None;
            }
        };
        let config: Config = toml::from_str(&contents).expect("failed to parse config");
        Some(config)
    }
}

pub fn create_config(package: &str) -> Result<(), String> {
    let filename = format!("{package}.toml");
    let mut path = PathBuf::from(TEMP_CONFIG_PATH);

    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|e| format!("failed to create temp config directory: {}", e))?;
    }

    path.push(filename);

    let template = format!(
        r#"# {package} configuration
update = "tagged" # no | live | tagged
build = "make"
install = "make install"
uninstall = "make uninstall"
dependencies = []
    "#
    );

    fs::write(path, template).map_err(|e| format!("failed to create config: {}", e))?;

    Ok(())
}

pub fn run_config_command(
    config_path: &Path,
    repo_path: &Path,
    command: ConfigCommand,
) -> Result<(), String> {
    let config = Config::new(config_path).ok_or("config not found".to_string())?;

    let is_build = matches!(command, ConfigCommand::Build);

    let cmd = match command {
        ConfigCommand::Build => config.build,
        ConfigCommand::Install => config.install,
        ConfigCommand::Uninstall => config.uninstall,
    };

    if let Some(c) = cmd {
        let mut parts = c.split_whitespace();
        let cmd_base = parts.next().ok_or("empty command".to_string())?;
        let args: Vec<&str> = parts.collect();

        let mut command = Command::new(cmd_base);
        command.args(&args).current_dir(repo_path);

        if is_build {
            if let Some((uid, gid, home, path)) = get_invoking_user_env() {
                command.env("HOME", home).env("PATH", path);
                unsafe {
                    command.pre_exec(move || {
                        nix::unistd::setgid(nix::unistd::Gid::from_raw(gid))?;
                        nix::unistd::setuid(nix::unistd::Uid::from_raw(uid))?;
                        Ok(())
                    });
                }
            }
        }

        let status = command
            .status()
            .map_err(|e| format!("failed to execute command: {}", e))?;

        if !status.success() {
            return Err(format!("command exited with non-zero status: {}", status));
        }
    }

    Ok(())
}

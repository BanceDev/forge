use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use libc;

pub const TEMP_CONFIG_PATH: &str = "/var/lib/forge/.tmp";

pub fn create_config(package: &str) -> Result<(), String> {
    let filename = format!("{package}.toml");
    let mut path = PathBuf::from(TEMP_CONFIG_PATH);

    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| {
            format!("failed to create temp config directory: {}", e)
        })?;
    }

    path.push(filename);

    let template = format!(
        r#"# {package} configuration
update = "tagged" # no | live | tagged
build = "make"
dependencies = []
install = "make install"
    "#
    );

    fs::write(path, template).map_err(|e| {
        format!("failed to create config: {}", e)
    })?;

    Ok(())
}

pub fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                size += metadata.len();
            } else if metadata.is_dir() {
                size += dir_size(&entry.path())?;
            }
        }
    }
    Ok(size)
}

pub fn get_editor() -> String {
    env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "nano".to_string())
}

pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0}
}

pub fn open_in_editor(editor: &str, file: &str) -> Result<(), String> {
    let status = Command::new(editor).arg(file).status().map_err(|e| {
        format!("failed to execute editor: {}", e)
    })?;

    if !status.success() {
        return Err(format!("editor exited with non-zero status: {}", status));
    }

    Ok(())
}

pub fn yn_prompt(prompt: &str) -> bool {
    print!("{} [y/n]: ", prompt);
    io::stdout().flush().unwrap();

    // Read input from user
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Normalize input
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "y" | "yes" | "" => true,
        _ => false,
    }
}

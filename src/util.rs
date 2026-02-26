use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;

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

pub fn get_invoking_user_env() -> Option<(u32, u32, String, String)> {
    let username = std::env::var("SUDO_USER").ok()?;
    if username.is_empty() {
        return None;
    }

    let user = nix::unistd::User::from_name(&username).ok()??;
    let uid = user.uid.as_raw();
    let gid = user.gid.as_raw();
    let home = user.dir.to_string_lossy().to_string();

    // Reconstruct a sane PATH for the user including common tool locations
    let path = format!(
        "{home}/.cargo/bin:{home}/.local/bin:/usr/local/bin:/usr/bin:/bin",
        home = home
    );

    Some((uid, gid, home, path))
}

pub fn open_in_editor(editor: &str, file: &str) -> Result<(), String> {
    let status = Command::new(editor)
        .arg(file)
        .status()
        .map_err(|e| format!("failed to execute editor: {}", e))?;

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

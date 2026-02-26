use crate::config::Config;
use crate::util::{
    TEMP_CONFIG_PATH, create_config, dir_size, get_editor, is_root, open_in_editor, yn_prompt,
};
use git2::Repository;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASE_REPO_PATH: &str = "/var/db/forge";
const BASE_CONFIG_PATH: &str = "/etc/forge/packages";

pub enum Action {
    Add { url: String },
    Update,
    Upgrade { packages: Vec<String> },
    Autoremove,
    Remove { packages: Vec<String> },
    List,
    Search { term: String },
    Clean { packages: Vec<String> },
    Show { package: String },
}

impl Action {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let cmd = args.get(1).ok_or("no command provided")?.as_str();

        match cmd {
            "add" => {
                let url = args.get(2).ok_or("add requires <repo>")?.clone();
                Ok(Action::Add { url })
            }
            "update" => Ok(Action::Update),
            "upgrade" => {
                let packages = args[2..].to_vec();
                Ok(Action::Upgrade { packages })
            }
            "autoremove" => Ok(Action::Autoremove),
            "remove" => {
                let packages = args[2..].to_vec();

                if packages.is_empty() {
                    Err("remove requires a package".into())
                } else {
                    Ok(Action::Remove { packages })
                }
            }
            "list" => Ok(Action::List),
            "search" => {
                let term = args.get(2).ok_or("search requires <term>")?.clone();
                Ok(Action::Search { term })
            }
            "clean" => {
                let packages = args[2..].to_vec();
                Ok(Action::Clean { packages })
            }
            "show" => {
                let package = args.get(2).ok_or("show requires <package>")?.clone();
                Ok(Action::Show { package })
            }
            _ => Err(format!("unknown command {}", cmd)),
        }
    }

    pub fn execute(self) -> Result<(), String> {
        match self {
            Action::Add { url } => add(url.as_str()),
            Action::Update => Ok(update()),
            Action::Upgrade { packages } => Ok(upgrade(packages)),
            Action::Autoremove => Ok(autoremove()),
            Action::Remove { packages } => remove(packages),
            Action::List => list(),
            Action::Search { term } => Ok(search(term)),
            Action::Clean { packages } => Ok(clean(packages)),
            Action::Show { package } => Ok(show(package)),
        }
    }
}

//=========================================
// command helpers
//=========================================

fn build(config_path: PathBuf, repo_path: PathBuf) -> Result<(), String> {
    let config = Config::new(config_path);
    if let Some(bld) = config.unwrap().build {
        let status = Command::new(bld)
            .current_dir(repo_path)
            .status()
            .map_err(|e| format!("failed to execute editor: {}", e))?;

        if !status.success() {
            return Err(format!("editor exited with non-zero status: {}", status));
        }
    } else {
        return Err("no build command found".to_string());
    }

    Ok(())
}

//=========================================
// commands
//=========================================

fn add(url: &str) -> Result<(), String> {
    if !is_root() {
        return Err("add must be run as root".to_string());
    }

    let repo_name = {
        let last_segment = url.rsplit('/').next().unwrap_or(url);
        last_segment.strip_suffix(".git").unwrap_or(last_segment)
    };
    let config_name = format!("{repo_name}.toml");

    println!("Creating config: {}", config_name);
    create_config(repo_name)?;

    let editor = get_editor();
    let config_temp = format!("{}/{}", TEMP_CONFIG_PATH, config_name);
    open_in_editor(&editor, &config_temp)?;

    let clone_path = PathBuf::from(BASE_REPO_PATH).join(repo_name);
    let repo = Repository::clone(url, &clone_path)
        .map_err(|e| format!("failed to clone {}: {}", repo_name, e))?;

    let mut config_path = PathBuf::from(BASE_CONFIG_PATH);
    if !config_path.exists() {
        fs::create_dir_all(&config_path)
            .map_err(|e| format!("failed to create config directory: {}", e))?;
    }

    config_path.push(config_name);

    fs::rename(config_temp, &config_path)
        .map_err(|e| format!("failed to place config in system directory: {}", e))?;

    println!(
        "New package initialized at: {}\n",
        repo.path().to_str().unwrap()
    );

    if yn_prompt("Run build and install commands?") {
        build(config_path, clone_path)?;
    }

    Ok(())
}

fn update() {
    println!("updating");
}

fn upgrade(packages: Vec<String>) {
    for (_, p) in packages.iter().enumerate() {
        println!("upgrading: {}", p);
    }
}

fn autoremove() {
    println!("autoremoving");
}

fn remove(packages: Vec<String>) -> Result<(), String> {
    if !is_root() {
        return Err("remove must be run as root".to_string());
    }

    println!("Checking dependencies...\n");
    let package_paths: Vec<(String, PathBuf, PathBuf)> = packages
        .into_iter()
        .map(|p| {
            let path = PathBuf::from(BASE_REPO_PATH).join(&p);
            let cfg_path = PathBuf::from(BASE_CONFIG_PATH).join(format!("{}.toml", p));
            if !path.exists() || !cfg_path.exists() {
                Err(format!("no installed package: {}", p))
            } else {
                Ok((p, path, cfg_path))
            }
        })
        .collect::<Result<_, _>>()?;

    println!(
        "Packages to remove ({}): {}\n",
        package_paths.len(),
        package_paths
            .iter()
            .map(|(p, _, _)| p.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let total_size: u64 = package_paths
        .iter()
        .map(|(_, path, _)| dir_size(path).unwrap_or(0))
        .sum();

    println!(
        "Total remove size: {:.2} MB\n",
        total_size as f64 / (1024.0 * 1024.0)
    );

    if yn_prompt("Proceed with removal?") {
        for (name, path, cfg_path) in package_paths {
            fs::remove_dir_all(&path).map_err(|e| format!("failed to remove {}: {}", name, e))?;
            fs::remove_file(&cfg_path).map_err(|e| format!("failed to remove {}: {}", name, e))?;
            println!("Removed {}", name);
        }
    }

    Ok(())
}

fn list() -> Result<(), String> {
    let config_path = Path::new(BASE_CONFIG_PATH);
    for entry in fs::read_dir(config_path)
        .map_err(|e| format!("failed to iterate package directory: {}", e))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(stem) = path.file_stem() {
                println!("{}", stem.to_string_lossy());
            }
        }
    }
    Ok(())
}

fn search(term: String) {
    println!("searching: {}", term);
}

fn clean(packages: Vec<String>) {
    for (_, p) in packages.iter().enumerate() {
        println!("cleaning: {}", p);
    }
}

fn show(package: String) {
    println!("showing {}", package);
}

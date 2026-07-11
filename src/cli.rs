use crate::{util, ui};

use std::{collections::HashMap, io};
use std::fs;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
pub struct VersionFile {
    pub versions: HashMap<String, VersionData>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VersionData {
    pub commit: String,
    pub install_path: String,
    pub source: String,
    pub version: String,
    pub description: Option<String>
}

impl VersionFile {
    pub fn from_json_str(data: &str) -> VersionFile {
        match serde_json::from_str(data) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error parsing from json!");
                eprintln!("{}", e);
                std::process::exit(1)
            }
        }
    }
}

// cli arguments
#[derive(clap::Parser)]
#[command(name  = "bashelle", version = env!("CARGO_PKG_VERSION"), arg_required_else_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub commands: Commands
}

#[derive(clap::Subcommand)]
pub enum Commands {
    #[command(about="Sync your dotfiles with a available release, and update them if theres new versions")]
    Update {},

    #[command(about="Prints your dotfiles versions installed. [--remote to fetch the latest release]")]
    Versions {
        #[arg(long)]
        remote: bool
    },
}

// cli functions 
pub fn check_updates() -> Vec<(String, VersionData)>{
    let update_json_str:String  =  remote_version_str();
    let lock_json_str:String    =  local_version_str();
    let mut updates_to_install: Vec<(String, VersionData)> = Vec::new();

    let updatefile: VersionFile = VersionFile::from_json_str(&update_json_str);
    let lockfile: VersionFile = VersionFile::from_json_str(&lock_json_str);
    
    for (name, update_version) in updatefile.versions {
        match lockfile.versions.get(&name) {
            Some(lock_version) => {
                
                if update_version.commit != lock_version.commit {
                    ui::print_update_info(&name, &update_version, false);
                    updates_to_install.push((name.clone(), update_version));
                }
            },
            None => {
                ui::print_update_info(&name, &update_version, true);
                updates_to_install.push((name.clone(), update_version));
            }
        }
    }

    return updates_to_install
}

pub fn backup(id: &str) -> Result<(), std::io::Error>{    
    let (source, target) = match dirs::config_dir() {
        Some(config ) => (
            config.join("bashelle"),
            config.join(format!("bashelle/.recovery/{}", id))
        ),        
        None => {
            eprintln!("config dir doesn't exist!");
            std::process::exit(1);
        }
    };

    util::copy_dir_all(&source, &target,&[".recovery", "wallpapers"])?;

    println!("{}", console::style("Backup created").bold());
    println!("└─ {} => {}", source.display(), target.display());
    Ok(())
}

pub fn install_dotfile(source: &str, install_path: &str) -> io::Result<()> {
    let request = ureq::get(source);
    let install_path = util::expand_user(install_path);

    if ! install_path.exists() {
        fs::create_dir_all(&install_path)?;
    }

    let mut body = match request.call() {
        Ok (r) => r.into_body(),
        Err(e) => {
            eprintln!("Error obtaining response from '{}'", source);
            eprintln!("{}", e);
            std::process::exit(1)
        }
    };

    let gz = GzDecoder::new(body.as_reader());

    let mut tar_archive = tar::Archive::new(gz);

    for entry in tar_archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;

        let mut components = entry_path.components();
        components.next();

        let install_path = install_path.join(components.as_path());

        if !components.as_path().as_os_str().is_empty() {            
            entry.unpack(install_path)?;
        }       
    }

    Ok(())
}

pub fn update_lock_file() -> io::Result<()> {
    let remote_version = remote_version_str();
    
    let lock_file = match dirs::config_dir() {
        Some(path) => path.join("bashelle/.versions-lock.json"),
        None => {
            eprintln!("config dir doesn't exist!");
            std::process::exit(1);
        }
    };

    fs::write(lock_file, remote_version)?;
    Ok(())
}

pub fn remote_version_str() -> String {
    let url = "https://github.com/Bashelle/bashelle/releases/latest/download/versions.json";
    let req = ureq::get(url);

    match req.call() {
        Ok(response ) => {
            let mut body = response.into_body();

            return body.read_to_string().unwrap();
        },
        Err(e) => {
            eprintln!("{}", console::style("Error fetching release").red().bold());
            eprintln!("{}", url);
            eprintln!("{}", e);
            
            std::process::exit(1);
        }
    }
}

pub fn local_version_str() -> String {
    let path = match dirs::config_dir() {
        Some(config) => config.join("bashelle/.versions-lock.json"),
        None => {
            eprintln!("config dir doesn't exist!");
            std::process::exit(1);
        }
    };

    return match fs::read_to_string(&path) {
        Ok(text) =>  text,
        Err(e) => {
            eprintln!("error loading: '{}'", path.display());
            eprintln!("detail: {}", e);
            eprintln!("{}", console::style("No versions to compare!").red().bold());
            std::process::exit(1);
        }
    };
}
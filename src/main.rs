mod cli;
mod ui;
mod util;

use std::time::Duration;

use clap::Parser;
use cli::VersionData;

fn main() {
    let cli = cli::Args::parse();

    match cli.commands {
        cli::Commands::Update {} => {
            let fetch_spinner = ui::spinner();
            fetch_spinner.set_message("Fetching releases...");
            fetch_spinner.enable_steady_tick(Duration::from_millis(30));

            let updates: Vec<(String, VersionData)> = cli::check_updates();
            
            if updates.is_empty() {
                println!("Everything is up on date! :)");
                std::process::exit(0);
            }
            
            fetch_spinner.finish_and_clear();
            
            if dialoguer::Confirm::new()
                .with_prompt(format!("Install ({}) updates now?", updates.len()))
                .interact().unwrap_or(false) 
            {
                
                let backup_name = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let install_spinner = ui::spinner();

                if let Err(e) = cli::backup(&backup_name.to_string()) {
                    util::log_error(&e.to_string(), "Backup failed!");
                    
                    if !dialoguer::Confirm::new()
                        .with_prompt("Continue anyways?")
                        .interact().unwrap_or(false) 
                    {
                       std::process::exit(0); 
                    }
                };
                
                install_spinner.enable_steady_tick(Duration::from_millis(30));

                for (i, (name, data)) in updates.iter().enumerate() {
                    let msg = format!("[{}/{}] Installing updates for '{}'", i+1, updates.len(), name);
                    install_spinner.set_message(msg);

                    if let Err(e) = cli::install_dotfile(&data.source, &data.install_path) {
                        util::log_error(&e.to_string(), "Error installing dotfile. Try again...");
                        std::process::exit(1);
                    }   
                }
                
                install_spinner.set_message("Updating versions-lock...");
                if let Err(e) = cli::update_lock_file() {
                    util::log_error(&e.to_string(), "Error updating lockfile, try again.");
                    std::process::exit(1);
                }
                
                install_spinner.finish_and_clear();
                println!("{} Bashelle has been updated.", console::style("").green());
            }
        },
        cli::Commands::Versions { remote } => {
            println!("{}",
                if remote {
                    let spinner = ui::spinner();
                    spinner.enable_steady_tick(std::time::Duration::from_millis(30)); 
                    spinner.set_message("Fetching remote 'version.json'...");
                    
                    cli::remote_version_str()
                } 
                else { cli::local_version_str() }
            );
        }
    }
}


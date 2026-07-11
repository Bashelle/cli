use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::VersionData;

pub fn spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
   
    spinner.set_style(
        ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template("{spinner:.cyan} {msg}").unwrap()
    );


    return spinner;
}

pub fn print_update_info(name: &str, data: &VersionData, new: bool) {
    println!("'{}' {}",
        console::style(&name).bold(),
        if new {
            console::style("(New)").bold()
        } else {
            console::style("(Update)").bold()
        }
    );
    
    if let Some(desc) = &data.description {                        
        println!("├─ description    {:?}", desc);
    }
    
    println!("├─ install_path   {:?}", data.install_path);
    println!("├─ from           {:?}", data.source);
    println!("├─ version        {:?}", data.version);
    println!("└─ commit         {:?}", data.hash);
    println!();
}
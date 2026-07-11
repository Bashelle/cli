use std::{env, fs, io, path::PathBuf};
use std::path::Path;

pub fn expand_user(path: &str) -> PathBuf {
    if !path.starts_with("~") {
        return PathBuf::from(path)    
    }    

    let home = env::var("HOME").unwrap(); // <- Assuming your pc is not broken...

    let expanded_path = path.replacen("~",home.as_str(), 1);

    return PathBuf::from(expanded_path)
}

pub fn copy_dir_all(src: impl AsRef<Path>, target: impl AsRef<Path>, exclude: &[&str]) -> io::Result<()> {
    fs::create_dir_all(&target)?;
 
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().into_string().unwrap();

        if !exclude.contains(&file_name.as_str()) {
            if file_type.is_file() {
                fs::copy(entry.path(), target.as_ref().join(entry.file_name()))?;
            }
            else {
                copy_dir_all(entry.path(), target.as_ref().join(entry.file_name()), exclude)?;
            }
        }
    }

    Ok(())
}

pub fn log_error(error: &str, message: &str) {
    println!("{}", console::style(message).bold().red());
    println!("└─ {}", error)
}
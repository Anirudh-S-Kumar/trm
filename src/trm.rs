use std::fs;
use std::{io::Error, path::PathBuf};
use chrono::{DateTime, Local};
use clap::Parser;

#[cfg(target_os = "linux")]
pub static DEFAULT_DIR: &str = "/var/tmp/trm_files";
#[cfg(target_os = "linux")]
pub static LOG_DIR: &str = "/var/log/trm";

#[cfg(target_os = "windows")]
pub static DEFAULT_DIR: &str = "C:\\Temp\\trm_files";
#[cfg(target_os = "windows")]
pub static LOG_DIR: &str = "C:\\ProgramData\\trm\\log";

#[cfg(target_os = "macos")]
pub static DEFAULT_DIR: &str = "/var/tmp/trm_files";
#[cfg(target_os = "macos")]
pub static LOG_DIR: &str = "/var/log/trm";

#[derive(Parser, Debug, Default)]
#[command(version, about = "trm - Temporary rm, a utility to reversibly remove your files", long_about=None)]
#[command(arg_required_else_help(true))]
pub struct Args{
    /// Files to delete
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<String>,

    /// Display full file paths or not
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Debug output
    #[arg(long)]
    pub debug: bool,

    /// Directory where to move
    #[arg(short, long, default_value_t = DEFAULT_DIR.to_string())]
    pub dir: String
}

struct _FileInfo{
    /// The original path from where the path was copied
    path: String,

    /// The datetime when it was moved
    moved_time: DateTime<Local>
}


pub fn setup_logging() -> Result<bool, Error>{
    if let Err(e) = std::fs::create_dir_all(LOG_DIR){
        eprintln!("Failed to create logging directory {}: {}", LOG_DIR, e);
        return Err(e);
    }

    let log_path = PathBuf::from(LOG_DIR);
    let history_path = log_path.join("history");

    if !history_path.try_exists().unwrap(){
        std::fs::File::create(history_path).unwrap();
    }

    Ok(true)
}

pub fn setup_directory(args:&Args) -> Result<PathBuf, Error>{
    let dir: String;
    let mut var_dir: String = String::new(); 

    #[cfg(target_os = "linux")]
    match std::env::var("XDG_DATA_HOME") {
        Ok(default_dir) => { var_dir = default_dir; }
        Err(_) => {}
    }

    // #[cfg(target_os = "windows")]
    // match std::env::var("APPDATA") {
    //     Ok(default_dir) => { var_dir = default_dir; }
    //     Err(_) => {}
    // }

    // #[cfg(target_os = "macos")]
    // match std::env::var("HOME") {
    //     Ok(home_dir) => { var_dir = format!("{}/Library/Application Support", home_dir); }
    //     Err(_) => {}
    // }

    if args.dir != DEFAULT_DIR {
        dir = args.dir.clone();
    } else if !var_dir.is_empty() {
        dir = var_dir;
    } else{
        dir = args.dir.clone();
    }

    let dir_path = match PathBuf::from(&dir).canonicalize(){
        Ok(dir) => dir,
        Err(_) => {
            if let Err(e) = fs::create_dir_all(&dir){
                eprintln!("Failed to create directory {}: {}", dir, e);
                return Err(e);
            }
            PathBuf::from(&dir)
        }
    };

    if args.debug {
        println!("Temporary Directory Path: {}", dir_path.display().to_string());
    }

    Ok(dir_path)
}

/// This does the following
/// 
/// 1. Create a info file, which stores the name and time at which it was moved here
/// 2. Move the file
pub fn move_files(args: &Args, dir_path: &PathBuf, files: &Vec<PathBuf>){
    for file in files{
        let full_path = match file.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to canonicalize path {}: {}", file.display(), e);
                return;
            }
        };

        let new_location = dir_path.join(full_path.strip_prefix("/").unwrap());

        if args.debug{
            println!("New location: {}", new_location.display());
        }

        // ensuring parent directories exist
        if let Some(parent) = new_location.parent(){
            if let Err(e) = fs::create_dir_all(parent){
                eprintln!("Failed to create directory {}: {}", parent.display(), e);
                return;
            }
        }
        

        match fs::rename(&full_path, &new_location) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to move files from {} to {}: {}", full_path.display(), new_location.display(), e);
                return;
            }
        }
    } 
}
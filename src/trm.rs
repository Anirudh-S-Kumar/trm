use crate::logging::{append_to_logs, FileInfo, OpType};
use clap::Parser;
use std::fs;
use std::{
    io::Error,
    path::PathBuf,
};

use crate::utils;

pub static DEFAULT_DIR: &str = "/var/tmp/trm_files";
pub static LOG_FILE: &str = "/var/tmp/trm.log";

#[derive(Parser, Debug, Default)]
#[command(version, about = "trm - Temporary rm, a utility to reversibly remove your files", long_about=None)]
#[command(arg_required_else_help(true))]
pub struct Args {
    /// Files to delete    
    #[arg(required_unless_present="list", num_args = 0..)]
    pub files: Vec<String>,

    /// Display full file paths or not
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Debug output
    #[arg(long)]
    pub debug: bool,

    // Recover files from the trash
    #[arg(short, long)]
    pub undo: bool,

    /// Display all files trashed under given directories.
    /// Takes current directory as default if no other directory given
    #[arg(short, long)]
    pub list: bool,

    /// Directory where to move
    #[arg(short, long, default_value_t = DEFAULT_DIR.to_string())]
    pub dir: String,
}



/// This does the following
///
/// 1. Create a info file, which stores the name and time at which it was moved here
/// 2. Move the file
pub fn move_files(args: &Args, dir_path: &PathBuf, files: &Vec<PathBuf>) {
    for file in files {
        let full_path = match file.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to canonicalize path {}: {}", file.display(), e);
                return;
            }
        };

        let mut new_location = dir_path.join(full_path.strip_prefix("/").unwrap());

        // ensuring parent directories exist
        if let Some(parent) = new_location.clone().parent() {
            // if name conflict exists, find the version number in a 2 step process
            // 1. Binary exponentiation to find upper limit
            // 2. Binary search to find actual number
            if new_location.exists() {
                let mut search_start = 1;
                let mut search_end = 1;
                let file_name = get_file_name!(file);
                while parent
                    .join(format!("{}_{}", file_name, &search_end.to_string()))
                    .exists()
                {
                    search_start = search_end;
                    search_end *= 2;
                }

                // Binary search
                while search_start < search_end {
                    let middle = (search_start + search_end) / 2;
                    let curr_file_name =
                        parent.join(format!("{}_{}", file_name, &middle.to_string()));

                    if curr_file_name.exists() {
                        search_start = middle + 1;
                    } else {
                        search_end = middle;
                    }
                }

                let new_file_name = format!("{}_{}", file_name, search_end);
                new_location.set_file_name(&new_file_name);

                if args.debug {
                    println!("New file name: {}", new_file_name);
                    println!("New file location: {}", new_location.display());
                }
            }

            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create directory {}: {}", parent.display(), e);
                std::process::exit(1);
            }
        }

        if args.debug {
            println!("New file location: {}", new_location.display());
        }

        match utils::move_content(&full_path, &new_location){ 
            Ok(_) => {
                if args.verbose {
                    println!("Successfully moved {} to trash", full_path.display());
                }
                append_to_logs(&FileInfo {
                    src: full_path.display().to_string(),
                    dst: new_location.display().to_string(),
                    operation: OpType::TRASH,
                    moved_time: chrono::offset::Local::now(),
                });
            }
            Err(e) => {
                eprintln!(
                    "Failed to move files from {} to {}: {}",
                    full_path.display(),
                    new_location.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }
}

pub fn list_delete_files(
    args: &Args,
    dir_path: &PathBuf,
    files: &mut Vec<PathBuf>,
    return_list: bool,
) -> Result<Vec<Vec<PathBuf>>, Error> {
    let cwd = std::env::current_dir().unwrap();
    if files.is_empty() {
        files.push(cwd.clone());
    }

    let mut deleted_files: Vec<Vec<PathBuf>> = vec![];

    for file in files.iter_mut() {
        let full_path = match file.canonicalize() {
            Ok(path) => path,
            Err(_) => cwd.clone().join(&file),
        };

        *file = dir_path.join(full_path.strip_prefix("/").unwrap());

        let sub_files: Vec<PathBuf> = match fs::read_dir(&file) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect(),
            Err(e) => {
                eprintln!("Failed to read directory {}: {}", file.display(), e);
                return Err(e);
            }
        };

        if return_list {
            deleted_files.push(sub_files.clone());
        }

        if sub_files.len() == 0 {
            println!("No files found under {}", file.display());
        } else {
            println!("{}:", file.display().to_string());
            utils::display_files(&sub_files, true);
        }
    }

    if args.debug {
        println!("Len of files: {}", files.len());
    }

    if return_list {
        return Ok(deleted_files);
    }

    Ok(vec![])
}

pub fn recover_files(args: &Args, dir_path: &PathBuf, files: &mut Vec<PathBuf>, from_trash: bool) {
    let cwd = std::env::current_dir().unwrap();

    for file in files.iter_mut() {
        let mut full_path = match file.canonicalize() {
            Ok(path) => path,
            Err(_) => cwd.clone().join(&file),
        };
        if !from_trash {
            *file = dir_path.join(full_path.strip_prefix("/").unwrap());
        } else {
            full_path = PathBuf::from("/").join(full_path.strip_prefix(dir_path).unwrap());
        }

        if file.exists() {
            match utils::move_content(&file, &full_path) {
                Ok(_) => {
                    if args.verbose {
                        println!(
                            "Successfully recovered file from trash to {}",
                            full_path.display()
                        );
                    }

                    append_to_logs(&FileInfo {
                        src: file.display().to_string(),
                        dst: full_path.display().to_string(),
                        operation: OpType::RESTORE,
                        moved_time: chrono::offset::Local::now(),
                    });
                }
                Err(e) => {
                    eprintln!(
                        "Failed to move files from {} to {}: {}",
                        file.display(),
                        full_path.display(),
                        e
                    );
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Unable to move {}: No such file exists", file.display());
        }
    }
}

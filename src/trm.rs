use crate::logging::{append_to_logs, FileInfo, OpType};
use chrono::Local;
use clap::{Parser, Subcommand};
use std::fs;
use std::{io::Error, path::PathBuf};

use crate::utils;


pub fn get_default_dir() -> String {
    let user =std::env::var("USER").unwrap_or("default".to_string());
    format!("/tmp/trm-{}", user)
}

pub fn get_log_file() -> String {
    let user =std::env::var("USER").unwrap_or("default".to_string());
    format!("/tmp/trm-{}.log", user)
}



#[derive(Parser, Debug, Default)]
#[command(version, about = "trm - Temporary rm, a utility to reversibly remove your files", long_about=None)]
#[command(subcommand_required = false, arg_required_else_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Files to delete    
    pub files: Vec<String>,

    /// Display full file paths or not
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Debug output
    #[arg(long)]
    pub debug: bool,

    /// Recover files from the trash
    #[arg(short, long)]
    pub undo: bool,

    /// All operation. 
    /// If combined with -l, will list all files in trash
    /// If combined with --undo, will recover all files from trash
    #[arg(short, long)]
    pub all: bool,


    /// Display all files trashed under given directories.
    /// Takes current directory as default if no other directory given
    #[arg(short, long)]
    pub list: bool,

    /// Directory where to move
    #[arg(short, long, default_value_t = get_default_dir())]
    pub dir: String,

}

#[derive(Subcommand, Debug)]
pub enum Commands{
    /// Shows history of all operations performed. By default it shows all the operations performed in current working directory
    #[command(about = "Shows history of all operations performed. For details on format for `before`, use --help", 
        long_about = "Shows history of all operations performed. By default it shows all the operations performed in current working directory

The `before` option supports the following syntax for specifying time
Example value could be `1hour 12min 5s`

* `nsec`, `ns` -- nanoseconds
* `usec`, `us` -- microseconds
* `msec`, `ms` -- milliseconds
* `seconds`, `second`, `sec`, `s`
* `minutes`, `minute`, `min`, `m`
* `hours`, `hour`, `hr`, `h`
* `days`, `day`, `d`
* `weeks`, `week`, `w`
* `months`, `month`, `M` -- defined as 30.44 days
* `years`, `year`, `y` -- defined as 365.25 days")]
    History {
        /// Show all the history
        #[arg(short, long)]
        all: bool,

        /// Show all changes before current time - given time
        #[arg(short, long, value_parser = humantime::parse_duration)]
        before: Option<std::time::Duration>,

        /// Directory to see history of. If no path specified, will show history in cwd
        #[arg(long, default_value_t = String::new())]
        path: String
    },

    
    /// Purge from trash and also corresponding logs. No files are deleted if --before is not provided
    Purge {
        /// Remove items before current time - given time. Follows same semantics as in history 
        #[arg(short, long, value_parser = humantime::parse_duration)]
        before: Option<std::time::Duration>,

        /// Confirm before purging
        #[arg(short, long, default_value_t = false)]
        quiet: bool,

        /// Purge all files in trash
        #[arg(short, long)]
        all: bool
    }
}

impl Args{
    pub fn validate(&self) -> Result<(), String>{
        // cases where files are not provided
        // 1. list is true
        // 2. some subcommand is provided
        // 3. undo and all are true
        let conditions = vec![
            self.list,
            self.undo && self.all,
            self.command.is_some()
        ];

        if !self.files.is_empty() && conditions.iter().any(|&x| x){
            return Err("Files must be provided".to_string());
        }

        // list, undo and all cannot be combined
        if self.list && self.undo && self.all{
            return Err("Cannot combine --list, --undo and --all. If you want to restore all files in trash, use --undo and --all".to_string());
        }

        Ok(())
    }
}



/// This does the following
///
/// 1. Create a info file, which stores the name and time at which it was moved here
/// 2. Move the file
pub fn move_files(args: &Args, dir_path: &PathBuf, files: &Vec<PathBuf>) {
    let mut src_files: Vec<String> = Vec::with_capacity(files.len());
    let mut dst_files: Vec<String> = Vec::with_capacity(files.len());

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

        match utils::move_content(&full_path, &new_location) {
            Ok(_) => {
                if args.verbose {
                    println!("Successfully moved {} to trash", full_path.display());
                }
                src_files.push(full_path.display().to_string());
                dst_files.push(new_location.display().to_string());
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

    if let Err(e) = append_to_logs(&FileInfo {
        src: src_files,
        dst: dst_files,
        operation: OpType::TRASH,
        moved_time: Local::now(),
    }) {
        eprintln!("Failed to append to logs: {}", e);
        std::process::exit(1);
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

        let sub_files: Vec<PathBuf> = fs::read_dir(&file)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

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

    let mut src_files: Vec<String> = Vec::with_capacity(files.len());
    let mut dst_files: Vec<String> = Vec::with_capacity(files.len());

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

                    src_files.push(file.display().to_string());
                    dst_files.push(full_path.display().to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Failed to move files from {} to {}: {}",
                        file.display(),
                        full_path.display(),
                        e
                    );
                }
            }
        } else {
            eprintln!("Unable to move {}: No such file exists", file.display());
        }
    }

    if src_files.is_empty(){
        return;
    }

    if let Err(e) = append_to_logs(&FileInfo {
        src: src_files,
        dst: dst_files,
        operation: OpType::RESTORE,
        moved_time: Local::now(),
    }) {
        eprintln!("Failed to append to logs: {}", e);
        std::process::exit(1);
    }
}

/// List all files in trash
pub fn list_all_files(dir_path: &PathBuf, return_list: bool) -> Vec<PathBuf>{
    let mut files: Vec<PathBuf> = Vec::with_capacity(1000);

    for file in walkdir::WalkDir::new(dir_path){
        files.push(file.unwrap().path().to_path_buf());
    }

    if return_list{
        return files;
    } else{
        utils::display_files(&files, false);
    }

    vec![]
}

/// Recover all files from trash
pub fn recover_all_files(args: &Args, dir_path: &PathBuf){
    let mut all_trash_files = list_all_files(&dir_path, true);
    recover_files(args, dir_path, &mut all_trash_files, true);
}
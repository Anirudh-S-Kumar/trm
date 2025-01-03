#[macro_use]
mod utils;
mod logging;
mod trm;

use chrono::Local;
use clap::Parser;
use logging::display_logs;
use std::{path::PathBuf, process::exit};
use trm::{Args, Commands, list_delete_files, recover_files, move_files};

fn main() {
    let args = Args::parse();

    if let Err(e) = args.validate(){
        eprintln!("Error validating args: {}", e);
        exit(1);
    }


    let mut files: Vec<PathBuf> = vec![];

    if args.debug {
        println!("Number of args received: {}", args.files.len());
    }

    for file in &args.files {
        files.push(PathBuf::from(file));
    }

    let dir_path = match utils::setup_directory(&args) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Could not create directory: {}", e);
            exit(1);
        }
    };

    if args.list && args.undo {
        let deleted_files = list_delete_files(&args, &dir_path, &mut files, true).unwrap();
        let mut flattened_files: Vec<PathBuf> = deleted_files.into_iter().flatten().collect();
        recover_files(&args, &dir_path, &mut flattened_files, true);
    } else if args.list {
        if let Err(e) = list_delete_files(&args, &dir_path, &mut files, false) {
            eprintln!("Error listing or deleting files: {}", e);
            exit(1);
        }
    } else if args.undo {
        recover_files(&args, &dir_path, &mut files, false);
    } else if let Some(Commands::History {all, before}) = args.command {
        if all{
            display_logs(logging::Filter::All);
        } else if let Some(before_duration) = before{
            let now = Local::now();
            let before_time = chrono::Duration::seconds(before_duration.as_secs() as i64);
            let cutoff = now - before_time;
            display_logs(logging::Filter::Before(cutoff));
        } 
        else{
            let cwd = std::env::current_dir().unwrap();
            display_logs(logging::Filter::Prefix(cwd));
        }
    } else {
        move_files(&args, &dir_path, &files);
    }
}

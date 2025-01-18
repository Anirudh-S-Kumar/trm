#[macro_use]
mod utils;
mod logging;
mod trm;

use chrono::{Local, Duration};
use clap::Parser;
use logging::{display_logs, purge_logs, Filter};
use std::{path::PathBuf, process::exit};
use trm::{list_all_files, recover_all_files, list_delete_files, move_files, recover_files, Args, Commands};

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
    } 
    else if args.list && args.all{
        list_all_files(&dir_path, false);
    }
    else if args.undo && args.all{
        recover_all_files(&args, &dir_path);
    }
    else if args.list {
        if let Err(e) = list_delete_files(&args, &dir_path, &mut files, false) {
            eprintln!("Error listing or deleting files: {}", e);
            exit(1);
        }
    } 
    else if args.undo {
        recover_files(&args, &dir_path, &mut files, false);
    } 
    else if let Some(Commands::History {all, before, path}) = args.command {
        if all{
            display_logs(Filter::All);
        } 
        else if let Some(before_duration) = before{
            let now = Local::now();
            let before_time = Duration::seconds(before_duration.as_secs() as i64);
            let cutoff = now - before_time;
            display_logs(Filter::Before(cutoff));
        } 
        else if !path.is_empty(){
            let path = PathBuf::from(path);
            if !path.exists(){
                eprintln!("Path does not exist: {}", path.display());
            }
            display_logs(Filter::Prefix(path));
        }

        else{
            let cwd = std::env::current_dir().unwrap();
            display_logs(Filter::Prefix(cwd));
        }
    } 
    else if let Some(Commands::Purge { before , quiet, all}) = args.command{
        let now = Local::now();

        if all{
            purge_logs(&args, now, quiet);
            return;
        }

        if let Some(before_duration) = before{
            let before_time = Duration::seconds(before_duration.as_secs() as i64);
            let cutoff = now - before_time;
            purge_logs(&args, cutoff, quiet);
        } else{
            eprintln!("No cutoff time provided");
            exit(1);
        }
    }
    else {
        move_files(&args, &dir_path, &files);
    }
}

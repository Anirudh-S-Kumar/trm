#[macro_use]
mod utils;
mod trm;
mod logging;


use clap::Parser;
use std::path::PathBuf;
use trm::*;

fn main() {
    let args = Args::parse();

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
            std::process::exit(1);
        }
    };

    if args.list && args.undo {
        let deleted_files = list_delete_files(&args, &dir_path, &mut files, true).unwrap();
        let mut flattened_files: Vec<PathBuf> = deleted_files.into_iter().flatten().collect();
        recover_files(&args, &dir_path, &mut flattened_files, true);
    } else if args.list {
        if let Err(e) = list_delete_files(&args, &dir_path, &mut files, false) {
            eprintln!("Error listing or deleting files: {}", e);
            std::process::exit(1);
        }
    } else if args.undo {
        recover_files(&args, &dir_path, &mut files, false);
    } else {
        move_files(&args, &dir_path, &files);
    }
}

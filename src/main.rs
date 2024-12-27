mod trm;

use trm::*;
use std::path::PathBuf;
use clap::Parser;



fn main() {
    let args = Args::parse();

    let mut files: Vec<PathBuf> = vec![];
    
    if args.debug{
        println!("Number of args received: {}", args.files.len());
    }

    for file in &args.files{
        files.push(PathBuf::from(file));
    }

    

    match setup_logging(){
        Ok(_) => {},
        Err(e) => {
            eprintln!("Could not setup logging: {}", e);
            std::process::exit(1);
        }
    }


    let dir_path = match setup_directory(&args){
        Ok(path) => path,
        Err(e) => {
            eprintln!("Could not create directory: {}", e);
            std::process::exit(1);
        }
    };

    if args.verbose{
        display_files(&files, false);
    }

    if args.list{
        list_delete_files(&args, &dir_path, &mut files);
    } else{
        move_files(&args, &dir_path, &files);
    }


}

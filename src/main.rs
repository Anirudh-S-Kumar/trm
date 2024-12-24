mod trm;

use trm::*;
use std::path::PathBuf;
use clap::Parser;
use lscolors::LsColors;



fn main() {
    let args = Args::parse_from(wild::args());
    let lscolors = LsColors::from_env().unwrap_or_default();

    let mut files: Vec<PathBuf> = vec![];
    
    if args.debug{
        println!("Number of args received: {}", args.files.len());
    }

    for file in &args.files{
        files.push(PathBuf::from(file));
    }

    if args.verbose {
        for file in &files {
            if let Some(style) = lscolors.style_for_path(&file){
                let crossterm_style = style.to_crossterm_style();
                println!("{}", crossterm_style.apply(file.display().to_string()));
            } else{
                println!("{}", file.display().to_string());
            }
        }
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


    move_files(&args, &dir_path, &files);

}

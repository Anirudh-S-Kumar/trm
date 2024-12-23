mod trm;

use trm::*;
use std::path::PathBuf;
use clap::Parser;
use glob::glob;
use lscolors::LsColors;



fn main() {
    let args = Args::parse_from(wild::args());
    let lscolors = LsColors::from_env().unwrap_or_default();

    let mut files: Vec<PathBuf> = Vec::new();
    
    if args.debug{
        println!("Number of args received: {}", args.files.len());
    }

    for file in &args.files{
        let escaped_file = glob::Pattern::escape(&file);
        

        for entry in match glob(&escaped_file) {
            Ok(paths) => paths,
            Err(e) => {
                eprintln!("Failed to read glob pattern: {}", e);
                continue;
            }
        } {
            match entry {
                Ok(path) => { files.push(path); }
                Err(e) => println!("{:?}", e)
            }
        }
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

}

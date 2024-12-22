use std::path::PathBuf;
use clap::Parser;
use glob::glob;
use lscolors::LsColors;


#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args{
    /// Files to delete
    files: Vec<String>,

    #[arg(short, long, default_value_t = false)]
    display: bool

}



fn main() {
    let args = Args::parse_from(wild::args());
    let lscolors = LsColors::from_env().unwrap_or_default();

    let mut files: Vec<PathBuf> = Vec::new();
    
    for file in args.files{
        let escaped_file = glob::Pattern::escape(&file);
        for entry in match glob(&escaped_file) {
            Ok(paths) => paths,
            Err(e) => {
                eprintln!("Failed to read glob pattern: {}", e);
                continue;
            }
        } {
            match entry {
                Ok(path) => files.push(path),
                Err(e) => println!("{:?}", e)
            }
        }
    }

    if args.display {
        for file in files {
            if let Some(style) = lscolors.style_for_path(&file){
                let crossterm_style = style.to_crossterm_style();
                println!("{}", crossterm_style.apply(file.display().to_string()));
            } else{
                println!("{}", file.display().to_string());
            }
        }
    }

}

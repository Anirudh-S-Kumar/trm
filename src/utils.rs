use lscolors::LsColors;
use std::{
    fs,
    io::{self, Error},
    path::PathBuf,
};
use term_grid::{Grid, GridOptions};

use crate::trm::{Args, get_default_dir};

#[macro_export]
macro_rules! get_file_name {
    ($path:expr) => {
        $path.file_name().unwrap().to_str().unwrap().to_string()
    };
}

pub fn move_content(original: &PathBuf, new_location: &PathBuf) -> Result<(), Error> {
    match fs::rename(&original, &new_location) {
        Ok(_) => (),
        Err(_) => {
            // do a copy and delete
            if original.is_file() {
                // if it is just a file, try normal copy and paste
                fs::copy(&original, &new_location)?;
                fs::remove_file(&original)?;
                return Ok(());
            }

            // perform copy and paste for a directory
            dircpy::copy_dir(&original, &new_location)?;
            fs::remove_dir_all(&original)?
        }
    }

    Ok(())
}

pub fn display_files(files: &Vec<PathBuf>, only_filename: bool) {
    let lscolors = LsColors::from_env().unwrap_or_default();
    let stdout_width = terminal_size::terminal_size_of(io::stdout())
        .map(|(w, _h)| w.0 as _)
        .unwrap_or(80);

    let file_names: Vec<String> = files
        .iter()
        .map(|file| {
            if let Some(style) = lscolors.style_for_path(&file) {
                let crossterm_style = style.to_crossterm_style();
                if only_filename {
                    return crossterm_style.apply(get_file_name!(file)).to_string();
                }
                crossterm_style
                    .apply(file.display().to_string())
                    .to_string()
            } else {
                if only_filename {
                    return get_file_name!(file);
                }
                file.display().to_string()
            }
        })
        .collect();

    let grid = Grid::new(
        file_names,
        GridOptions {
            filling: term_grid::Filling::Spaces(2),
            direction: term_grid::Direction::TopToBottom,
            width: stdout_width,
        },
    );

    println!("{grid}");
}

pub fn setup_directory(args: &Args) -> Result<PathBuf, Error> {
    let dir: String;
    let mut var_dir: String = String::new();

    match std::env::var("XDG_DATA_HOME") {
        Ok(default_dir) => {
            var_dir = default_dir;
        }
        Err(_) => {}
    }

    if args.dir != get_default_dir() {
        dir = args.dir.clone();
    } else if !var_dir.is_empty() {
        dir = var_dir;
    } else {
        dir = args.dir.clone();
    }

    let dir_path = match PathBuf::from(&dir).canonicalize() {
        Ok(dir) => dir,
        Err(_) => {
            if let Err(e) = fs::create_dir_all(&dir) {
                eprintln!("Failed to create directory {}: {}", dir, e);
                return Err(e);
            }
            PathBuf::from(&dir)
        }
    };

    if args.debug {
        println!(
            "Temporary Directory Path: {}",
            dir_path.display().to_string()
        );
    }

    Ok(dir_path)
}

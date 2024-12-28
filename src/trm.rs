use chrono::{DateTime, Local};
use clap::Parser;
use lscolors::LsColors;
use std::fs;
use std::{
    io::{self, Error},
    path::PathBuf,
};
use term_grid::{Grid, GridOptions};

pub static DEFAULT_DIR: &str = "/var/tmp/trm_files";
pub static LOG_DIR: &str = "/var/log/trm";

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

struct _FileInfo {
    /// The original path from where the path was copied
    path: String,

    /// The datetime when it was moved
    moved_time: DateTime<Local>,
}

macro_rules! get_file_name {
    ($path:expr) => {
        $path.file_name().unwrap().to_str().unwrap().to_string()
    };
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

pub fn setup_logging() -> Result<bool, Error> {
    if let Err(e) = std::fs::create_dir_all(LOG_DIR) {
        eprintln!("Failed to create logging directory {}: {}", LOG_DIR, e);
        return Err(e);
    }

    let log_path = PathBuf::from(LOG_DIR);
    let history_path = log_path.join("history");

    if !history_path.try_exists().unwrap() {
        std::fs::File::create(history_path).unwrap();
    }

    Ok(true)
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

    if args.dir != DEFAULT_DIR {
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

/// This does the following
///
/// 1. Create a info file, which stores the name and time at which it was moved here
/// 2. Move the file
/// TODO: Refactor the logic for finding new file path and transfer the move logic back to the main file
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

        match fs::rename(&full_path, &new_location) {
            Ok(_) => {
                if args.verbose {
                    println!("Successfully moved {} to trash", full_path.display());
                }
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
            display_files(&sub_files, true);
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
            match fs::rename(&file, &full_path) {
                Ok(_) => {
                    if args.verbose {
                        println!(
                            "Successfully recovered file from trash to {}",
                            full_path.display()
                        );
                    }
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

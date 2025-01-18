use crate::trm::{Args, get_log_file};

use chrono::{DateTime, Local};

use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions}, io::{self, BufRead, BufReader, Error, Write}, path::{Path, PathBuf}, process::exit
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OpType {
    TRASH,
    RESTORE,
}

impl OpType{
    pub fn to_string(&self) -> String{
        match &self{
            Self::TRASH => String::from("Trash"),
            Self::RESTORE => String::from("Restore")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    /// The original path from where the path was moved
    pub src: Vec<String>,

    /// The path where the file ended up in  
    pub dst: Vec<String>,

    /// Type of operation
    pub operation: OpType,

    /// The datetime when it was moved
    pub moved_time: DateTime<Local>,
}

pub fn append_to_logs(info: &FileInfo) -> Result<(), Error> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(get_log_file())
        .unwrap();

    let mut writer = io::BufWriter::new(file);
    let serialized_info = serde_json::to_string(info)?;
    writeln!(writer, "{}", serialized_info)?;
    writer.flush()?;
    Ok(())
}

fn generate_table() -> Table{
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Time", "Operation", "Source", "Destination"]);
    table
}
/// Filter is used for filtering the logs based on what we want
pub enum Filter{
    All,
    Prefix(PathBuf),
    Before(DateTime<Local>)
}

fn read_logs(filter: Filter) -> Vec<FileInfo> {
    let file = match File::open(get_log_file()){
        Ok(file ) => file,
        Err(e) => {
            eprintln!("Unable to open log file {}: {}", get_log_file(), e);
            exit(1);
        }
    };

    let reader = BufReader::new(file);
    let mut logs: Vec<FileInfo> = vec![];
    
    for line in reader.lines() {
        let line = match line{
            Ok(line) => line,
            Err(e) => {
                eprintln!("Unable to parse log: {}", e);
                exit(1);
            }
        };
        if let Ok(log) = serde_json::from_str::<FileInfo>(&line) {
            match &filter{
                Filter::All => {
                    logs.push(log);
                }
                Filter::Prefix(prefix) => {
                    if log.operation == OpType::TRASH{
                        if log.src.iter().any(|s| PathBuf::from(s).parent() == Some(prefix)){
                            logs.push(log);
                        }
                    } else{
                        if log.dst.iter().any(|s| PathBuf::from(s).parent() == Some(prefix)){
                            logs.push(log);
                        }
                    }
                }
                Filter::Before(cutoff) => {
                    if log.moved_time < *cutoff{
                        logs.push(log);
                    } else{
                        break;
                    }
                }
            }
        }
    }

    logs
}

pub fn display_logs(filter: Filter){
    let mut table = generate_table();
    let logs = read_logs(filter);

    if logs.is_empty(){
        eprintln!("No history to show");
        exit(1);
    }


    for log in logs {
        table.add_row(vec![
            log.moved_time.to_rfc2822(),
            log.operation.to_string(),
            log.src.join("\n"),
            log.dst.join("\n")
            ]
        );
    }

    println!("{}", table);
}

/// Purge old files in trash and also remove corresponding entries in log
pub fn purge_logs(args: &Args, cutoff: DateTime<Local>, quiet: bool){
    let file = match File::open(get_log_file()){
        Ok(file ) => file,
        Err(e) => {
            eprintln!("Unable to open log file {}: {}", get_log_file(), e);
            exit(1);
        }
    };

    let mut to_be_deleted_files: Vec<PathBuf> = vec![];
    let mut new_logs: Vec<FileInfo> = vec![];

    let reader = BufReader::new(file);
    for line in reader.lines(){
        let line = match line{
            Ok(line) => line,
            Err(e) => {
                eprintln!("Unable to parse log: {}", e);
                exit(1);
            }    
        };
        if let Ok(log) = serde_json::from_str::<FileInfo>(&line) {
            if log.moved_time < cutoff{
                if log.operation == OpType::RESTORE{
                    continue;
                } 
                for dst in log.dst{
                    let dst = PathBuf::from(dst);
                    if !dst.exists(){
                        if args.verbose{
                            println!("Path {} does not exist. Skipping", dst.display());
                        }
                        continue;
                    }

                    to_be_deleted_files.push(dst);

                }
            } else{
                new_logs.push(log);
            }
        }
    }

    if !quiet && !to_be_deleted_files.is_empty(){
        let mut input = String::new();
        for file in &to_be_deleted_files{
            println!("{}", file.display());
        }
        print!("The above files will be deleted. Do you want to continue? [y/N]: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y"{
            println!("Aborting");
            return;
        }
    }

    // deleting the files
    for dst in &to_be_deleted_files{
        // Delete the file/directory
        let mut curr_parent = dst.parent().unwrap();
        if dst.is_dir(){
            if let Err(e) = fs::remove_dir_all(&dst){
                eprintln!("Error deleting directory {}: {}", dst.display(), e);
            } else if args.verbose{
                println!("Removed {}", dst.display());
            }
        }
        else{
            if let Err(e) = fs::remove_file(&dst){
                eprintln!("Error deleting file {}: {}", dst.display(), e);
            } else if args.verbose{
                println!("Removed {}", dst.display());
            }
        }
        while curr_parent.exists(){
            let new_parent = curr_parent.parent().unwrap_or_else(|| Path::new(""));
            if let Err(_) = fs::remove_dir(curr_parent){
                break;
            }
            curr_parent = new_parent;
        }
    }

    // write new log to file
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_log_file())
        .unwrap();

    let mut writer = io::BufWriter::new(file);
    for log in new_logs{
        let serialized_info = serde_json::to_string(&log).unwrap();
        writeln!(writer, "{}", serialized_info).unwrap();
    }
    writer.flush().unwrap();

}
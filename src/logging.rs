use crate::trm::LOG_FILE;

use chrono::{DateTime, Local};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions}, io::{self, BufRead, BufReader, Error, Write}, path::PathBuf, process::exit
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
        .open(LOG_FILE)
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
    let file = match File::open(LOG_FILE){
        Ok(file ) => file,
        Err(e) => {
            eprintln!("Unable to open log file {}: {}", LOG_FILE, e);
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
                Filter::Before(date_time) => {
                    if log.moved_time < *date_time{
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



// pub fn _read_cwd_logs(){
//     let logs = read_logs(Filter::All);
//     let mut table = generate_table(); 

// }